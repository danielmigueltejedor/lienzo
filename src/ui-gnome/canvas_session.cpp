#include "ui-gnome/canvas.hpp"
#include "ui-gnome/canvas_internal.hpp"
#include "ui-gnome/brush_tips.hpp"
#include "ui-gnome/tools/selection_controller.hpp"
#include "ui-gnome/tools/text_controller.hpp"
#include "ui-gnome/tools/path_controller.hpp"
#include "ui-gnome/tools/retouch_controller.hpp"

#include "core/layer_metadata.hpp"
#include "core/magnetic_lasso.hpp"
#include "core/pixel_tools.hpp"
#include "core/rect_utils.hpp"
#include "core/stroke_stabilizer.hpp"
#include "render/compositor.hpp"

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <memory>
#include <optional>
#include <string>
#include <vector>

namespace lienzo::gnome {

// History, clipboard, crop commit, and the active editable layer.


patchy::Layer* find_last_pixel_layer(
    std::vector<patchy::Layer>& layers) {
  for (auto it = layers.rbegin();
       it != layers.rend();
       ++it) {
    if (it->kind() == patchy::LayerKind::Group) {
      if (auto* child =
              find_last_pixel_layer(it->children());
          child != nullptr) {
        return child;
      }
    }

    if (it->kind() == patchy::LayerKind::Pixel) {
      return &*it;
    }
  }

  return nullptr;
}

std::optional<patchy::LayerId> editing_layer(
    CanvasState* state) {
  auto active =
      state->document->active_layer_id();

  if (active.has_value()) {
    auto* layer =
        state->document->find_layer(*active);

    if (
        layer != nullptr &&
        layer->kind() == patchy::LayerKind::Pixel) {
      return active;
    }
  }

  auto* layer =
      find_last_pixel_layer(
          state->document->layers());

  if (layer == nullptr) {
    return std::nullopt;
  }

  state->document->set_active_layer(
      layer->id());

  return layer->id();
}

void notify_document_changed(
    CanvasState* state) {
  if (state->document_changed_callback) {
    state->document_changed_callback();
  }
}

std::size_t layer_pixel_bytes(
    const std::vector<patchy::Layer>& layers) {
  std::size_t bytes = 0;

  for (const auto& layer : layers) {
    bytes += layer.pixels().byte_size();

    if (layer.mask().has_value()) {
      bytes += layer.mask()->pixels.byte_size();
    }

    bytes += layer_pixel_bytes(layer.children());
  }

  return bytes;
}

void trim_history(
    std::vector<HistorySnapshot>& stack,
    std::size_t bytes) {
  constexpr std::size_t kMaxHistory = 32;
  constexpr std::size_t kMaxBytes =
      512ull * 1024ull * 1024ull;

  std::size_t limit = kMaxHistory;

  if (bytes > 0) {
    limit = std::min(
        kMaxHistory,
        std::max(
            std::size_t{1},
            kMaxBytes / bytes));
  }

  while (stack.size() > limit) {
    stack.erase(stack.begin());
  }
}

void push_history(
    CanvasState* state,
    const char* label) {
  state->undo_stack.push_back(
      HistorySnapshot{
          *state->document,
          state->current_label});

  state->current_label =
      label != nullptr
          ? label
          : tool_name(state->tool);

  trim_history(
      state->undo_stack,
      layer_pixel_bytes(
          state->document->layers()));

  state->redo_stack.clear();
}

void undo_document(
    CanvasState* state) {
  if (state->undo_stack.empty()) {
    return;
  }

  state->redo_stack.push_back(
      HistorySnapshot{
          *state->document,
          state->current_label});

  trim_history(
      state->redo_stack,
      layer_pixel_bytes(
          state->document->layers()));

  state->current_label =
      state->undo_stack.back().label;

  *state->document =
      std::move(
          state->undo_stack.back().document);

  state->undo_stack.pop_back();

  state->view_initialized = false;

  refresh_canvas(state);
  notify_document_changed(state);
}

void redo_document(
    CanvasState* state) {
  if (state->redo_stack.empty()) {
    return;
  }

  state->undo_stack.push_back(
      HistorySnapshot{
          *state->document,
          state->current_label});

  state->current_label =
      state->redo_stack.back().label;

  *state->document =
      std::move(
          state->redo_stack.back().document);

  state->redo_stack.pop_back();

  state->view_initialized = false;

  refresh_canvas(state);
  notify_document_changed(state);
}

void copy_active_layer(
    CanvasState* state) {
  const auto active =
      state->document->active_layer_id();

  if (!active.has_value()) {
    return;
  }

  const auto* layer =
      state->document->find_layer(
          *active);

  if (
      layer == nullptr ||
      layer->kind() !=
          patchy::LayerKind::Pixel ||
      layer->pixels().empty()) {
    return;
  }

  state->clipboard =
      CanvasState::ClipboardLayer{
          layer->pixels(),
          layer->bounds(),
          layer->name()};
}

void cut_active_layer(
    CanvasState* state) {
  const auto active =
      state->document->active_layer_id();

  if (!active.has_value()) {
    return;
  }

  copy_active_layer(state);

  if (!state->clipboard.has_value()) {
    return;
  }

  push_history(state);

  if (
      state->document->remove_layer(
          *active)) {
    refresh_canvas(state);
    notify_document_changed(state);
  }
}

void paste_layer(
    CanvasState* state) {
  if (!state->clipboard.has_value()) {
    return;
  }

  push_history(state);

  const auto& copied =
      *state->clipboard;

  patchy::Layer layer(
      state->document->allocate_layer_id(),
      copied.name + " copy",
      copied.pixels);

  layer.set_bounds(
      copied.bounds);

  state->document->add_layer(
      std::move(layer));

  refresh_canvas(state);
  notify_document_changed(state);
}

bool commit_crop(
    CanvasState* state) {
  if (
      state->tool != Tool::Crop ||
      !state->crop_session_active ||
      state->crop_rect.empty()) {
    return false;
  }

  push_history(state);

  if (
      !patchy::crop_document(
          *state->document,
          state->crop_rect)) {
    return false;
  }

  state->crop_session_active = false;
  state->view_initialized = false;

  refresh_canvas(state);
  notify_document_changed(state);

  return true;
}

bool cancel_crop(
    CanvasState* state) {
  if (!state->crop_session_active) {
    return false;
  }

  state->crop_session_active = false;

  gtk_widget_queue_draw(
      GTK_WIDGET(state->area));

  return true;
}

std::vector<HistoryEntry> history_rows(
    const CanvasState* state) {
  std::vector<HistoryEntry> rows;

  rows.reserve(
      state->undo_stack.size() +
      state->redo_stack.size() +
      1);

  for (const auto& step : state->undo_stack) {
    rows.push_back(
        HistoryEntry{step.label, false});
  }

  rows.push_back(
      HistoryEntry{state->current_label, true});

  for (
      auto it = state->redo_stack.rbegin();
      it != state->redo_stack.rend();
      ++it) {
    rows.push_back(
        HistoryEntry{it->label, false});
  }

  return rows;
}

void restore_history(
    CanvasState* state,
    int index) {
  if (index < 0) {
    return;
  }

  const int current =
      static_cast<int>(
          state->undo_stack.size());

  const int last =
      current +
      static_cast<int>(
          state->redo_stack.size());

  if (index > last || index == current) {
    return;
  }

  if (index < current) {
    while (
        static_cast<int>(
            state->undo_stack.size()) >
        index) {
      state->redo_stack.push_back(
          HistorySnapshot{
              *state->document,
              state->current_label});

      state->current_label =
          state->undo_stack.back().label;

      *state->document =
          std::move(
              state->undo_stack.back()
                  .document);

      state->undo_stack.pop_back();
    }
  } else {
    const int steps = index - current;

    for (int step = 0; step < steps; ++step) {
      redo_document(state);
    }

    return;
  }

  state->view_initialized = false;
  refresh_canvas(state);
  notify_document_changed(state);
}

void align_active_to_canvas(
    CanvasState* state,
    patchy::AlignEdge edge) {
  const auto active =
      state->document->active_layer_id();

  if (!active.has_value()) {
    return;
  }

  if (patchy::layer_effectively_locks_position(
          state->document->layers(),
          *active)) {
    return;
  }

  auto* layer =
      state->document->find_layer(*active);

  if (layer == nullptr) {
    return;
  }

  const auto bounds = layer->bounds();

  if (bounds.empty()) {
    return;
  }

  const auto deltas =
      patchy::compute_align_deltas(
          {bounds},
          patchy::Rect::from_size(
              state->document->width(),
              state->document->height()),
          edge);

  if (
      deltas.empty() ||
      deltas.front().is_zero()) {
    return;
  }

  push_history(state, "Alinear");

  auto moved = layer->bounds();
  moved.x += deltas.front().dx;
  moved.y += deltas.front().dy;
  layer->set_bounds(moved);

  refresh_canvas(state);
  notify_document_changed(state);
}

void add_centered_guide(
    CanvasState* state,
    patchy::GuideOrientation orientation) {
  const bool vertical =
      orientation ==
      patchy::GuideOrientation::Vertical;

  const std::int32_t limit =
      vertical
          ? state->document->width()
          : state->document->height();

  if (limit <= 0) {
    return;
  }

  push_history(state, "Guía");

  state->document->guides().push_back(
      patchy::DocumentGuide{
          orientation,
          (limit / 2) * 32});

  gtk_widget_queue_draw(
      GTK_WIDGET(state->area));

  notify_document_changed(state);
}

bool scale_active_layer(
    CanvasState* state,
    int width_percent,
    int height_percent) {
  width_percent =
      std::clamp(width_percent, 1, 800);

  height_percent =
      std::clamp(height_percent, 1, 800);

  if (
      width_percent == 100 &&
      height_percent == 100) {
    return false;
  }

  const auto active =
      state->document->active_layer_id();

  if (!active.has_value()) {
    return false;
  }

  auto* layer =
      state->document->find_layer(*active);

  if (
      layer == nullptr ||
      layer->kind() != patchy::LayerKind::Pixel ||
      (layer->lock_flags() &
       patchy::kLayerLockImagePixels) != 0) {
    return false;
  }

  const auto& source =
      std::as_const(*layer).pixels();

  if (source.empty()) {
    return false;
  }

  const int width =
      std::clamp(
          static_cast<int>(
              std::lround(
                  source.width() *
                  (width_percent / 100.0))),
          1,
          8192);

  const int height =
      std::clamp(
          static_cast<int>(
              std::lround(
                  source.height() *
                  (height_percent / 100.0))),
          1,
          8192);

  if (
      width == source.width() &&
      height == source.height()) {
    return false;
  }

  auto scaled =
      patchy::scale_pixels_resampled(
          source,
          width,
          height);

  push_history(state, "Transformar");

  auto bounds = layer->bounds();
  bounds.width = width;
  bounds.height = height;
  layer->set_pixels(std::move(scaled));
  layer->set_bounds(bounds);

  refresh_canvas(state);
  notify_document_changed(state);

  return true;
}

void flip_active_layer(
    CanvasState* state,
    bool horizontal) {
  const auto active =
      state->document->active_layer_id();

  if (!active.has_value()) {
    return;
  }

  auto* layer =
      state->document->find_layer(*active);

  if (
      layer == nullptr ||
      (layer->lock_flags() &
       patchy::kLayerLockImagePixels) != 0) {
    return;
  }

  push_history(state, "Voltear");

  if (horizontal) {
    (void)patchy::flip_layer_horizontal(
        *state->document,
        *active);
  } else {
    (void)patchy::flip_layer_vertical(
        *state->document,
        *active);
  }

  refresh_canvas(state);
  notify_document_changed(state);
}

void capture_mixer_snapshot(
    CanvasState* state) {
  state->mixer_snapshot.clear();
  state->mixer_width = 0;
  state->mixer_height = 0;

  patchy::begin_mixer_brush_stroke(
      state->mixer_state);

  const auto active = editing_layer(state);

  if (!active.has_value()) {
    return;
  }

  const auto* layer =
      std::as_const(*state->document)
          .find_layer(*active);

  if (
      layer == nullptr ||
      layer->kind() != patchy::LayerKind::Pixel) {
    return;
  }

  const auto& pixels = layer->pixels();

  if (
      pixels.empty() ||
      pixels.format() != patchy::PixelFormat::rgba8()) {
    return;
  }

  const auto bytes = pixels.data();

  state->mixer_origin_x = layer->bounds().x;
  state->mixer_origin_y = layer->bounds().y;
  state->mixer_width = pixels.width();
  state->mixer_height = pixels.height();
  state->mixer_snapshot.assign(
      bytes.begin(),
      bytes.end());
}

patchy::EditColor sample_mixer_pickup(
    const CanvasState* state,
    double x,
    double y,
    int brush_size) {
  const double radius =
      std::max(
          0.5,
          static_cast<double>(
              std::max(1, brush_size)) /
              2.0);

  constexpr int kGrid = 9;
  double sum_r = 0.0;
  double sum_g = 0.0;
  double sum_b = 0.0;
  double sum_alpha = 0.0;
  int taps = 0;

  for (int gy = 0; gy < kGrid; ++gy) {
    for (int gx = 0; gx < kGrid; ++gx) {
      const double offset_x =
          (static_cast<double>(gx) /
               (kGrid - 1) -
           0.5) *
          2.0;

      const double offset_y =
          (static_cast<double>(gy) /
               (kGrid - 1) -
           0.5) *
          2.0;

      if (
          offset_x * offset_x +
              offset_y * offset_y >
          1.0) {
        continue;
      }

      ++taps;

      const auto sample_x =
          static_cast<std::int32_t>(
              std::lround(
                  x + offset_x * radius));

      const auto sample_y =
          static_cast<std::int32_t>(
              std::lround(
                  y + offset_y * radius));

      const auto local_x =
          sample_x - state->mixer_origin_x;

      const auto local_y =
          sample_y - state->mixer_origin_y;

      if (
          state->mixer_snapshot.empty() ||
          local_x < 0 ||
          local_y < 0 ||
          local_x >= state->mixer_width ||
          local_y >= state->mixer_height) {
        continue;
      }

      const auto* pixel =
          state->mixer_snapshot.data() +
          (static_cast<std::size_t>(local_y) *
               static_cast<std::size_t>(
                   state->mixer_width) +
           static_cast<std::size_t>(local_x)) *
              4U;

      const double alpha =
          static_cast<double>(pixel[3]) / 255.0;

      sum_r += pixel[0] * alpha;
      sum_g += pixel[1] * alpha;
      sum_b += pixel[2] * alpha;
      sum_alpha += alpha;
    }
  }

  if (taps == 0 || sum_alpha <= 0.0) {
    return {0, 0, 0, 0};
  }

  const auto to_byte = [](double value) {
    return static_cast<std::uint8_t>(
        std::clamp(
            std::lround(value),
            0L,
            255L));
  };

  return {
      to_byte(sum_r / sum_alpha),
      to_byte(sum_g / sum_alpha),
      to_byte(sum_b / sum_alpha),
      to_byte(sum_alpha * 255.0 / taps)};
}

void install_mixer_provider(
    CanvasState* state) {
  if (state->tool != Tool::MixerBrush) {
    state->edit_options.dab_primary_provider =
        nullptr;

    return;
  }

  capture_mixer_snapshot(state);

  state->edit_options.dab_primary_provider =
      [state](
          double x,
          double y,
          const patchy::EditColor& loaded) {
        const auto picked =
            sample_mixer_pickup(
                state,
                x,
                y,
                state->edit_options.brush_size);

        return patchy::mixer_brush_dab_color(
            state->mixer_state,
            x,
            y,
            state->edit_options.brush_size,
            loaded,
            picked,
            state->mixer_wet,
            state->mixer_load,
            state->mixer_mix);
      };
}

void clear_mixer_provider(
    CanvasState* state) {
  state->edit_options.dab_primary_provider =
      nullptr;
}

}  // namespace lienzo::gnome
