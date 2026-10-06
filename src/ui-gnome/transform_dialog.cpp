#include "ui-gnome/transform_dialog.hpp"

#include <adwaita.h>

namespace lienzo::gnome {

namespace {

struct TransformDialog {
  CanvasView canvas;
  AdwSpinRow* width{};
  AdwSpinRow* height{};
  AdwDialog* dialog{};
};

void close_later(AdwDialog* dialog) {
  g_object_ref(dialog);

  g_idle_add(
      [](gpointer data) -> gboolean {
        auto* target = ADW_DIALOG(data);
        adw_dialog_close(target);
        g_object_unref(target);
        return G_SOURCE_REMOVE;
      },
      dialog);
}

void apply_transform(
    GtkWidget*,
    gpointer data) {
  auto* dialog = static_cast<TransformDialog*>(data);

  if (dialog->canvas.scale_active_layer) {
    dialog->canvas.scale_active_layer(
        static_cast<int>(adw_spin_row_get_value(dialog->width)),
        static_cast<int>(adw_spin_row_get_value(dialog->height)));
  }

  close_later(dialog->dialog);
}

void flip_transform(
    GtkWidget* button,
    gpointer data) {
  auto* dialog = static_cast<TransformDialog*>(data);

  if (dialog->canvas.flip_active_layer) {
    const bool horizontal =
        g_object_get_data(G_OBJECT(button), "lienzo-flip-horizontal") != nullptr;

    dialog->canvas.flip_active_layer(horizontal);
  }

  close_later(dialog->dialog);
}

}  // namespace

void present_layer_transform(
    GtkWidget* host,
    const CanvasView& canvas) {
  auto* dialog = new TransformDialog;
  dialog->canvas = canvas;

  GtkWidget* page = adw_preferences_page_new();
  GtkWidget* group = adw_preferences_group_new();
  adw_preferences_group_set_title(
      ADW_PREFERENCES_GROUP(group),
      "Escala");
  adw_preferences_page_add(
      ADW_PREFERENCES_PAGE(page),
      ADW_PREFERENCES_GROUP(group));

  dialog->width = ADW_SPIN_ROW(adw_spin_row_new_with_range(1, 800, 1));
  adw_spin_row_set_value(dialog->width, 100);
  adw_preferences_row_set_title(ADW_PREFERENCES_ROW(dialog->width), "Ancho %");
  adw_preferences_group_add(ADW_PREFERENCES_GROUP(group), GTK_WIDGET(dialog->width));

  dialog->height = ADW_SPIN_ROW(adw_spin_row_new_with_range(1, 800, 1));
  adw_spin_row_set_value(dialog->height, 100);
  adw_preferences_row_set_title(ADW_PREFERENCES_ROW(dialog->height), "Alto %");
  adw_preferences_group_add(ADW_PREFERENCES_GROUP(group), GTK_WIDGET(dialog->height));

  dialog->dialog = adw_dialog_new();
  GtkWidget* window = GTK_WIDGET(dialog->dialog);
  adw_dialog_set_title(dialog->dialog, "Transformar capa");
  adw_dialog_set_content_width(dialog->dialog, 420);
  adw_dialog_set_content_height(dialog->dialog, 420);

  GtkWidget* toolbar = adw_toolbar_view_new();
  GtkWidget* header = adw_header_bar_new();
  GtkWidget* horizontal = gtk_button_new_with_label("Voltear H");
  GtkWidget* vertical = gtk_button_new_with_label("Voltear V");
  g_object_set_data(G_OBJECT(horizontal), "lienzo-flip-horizontal", GINT_TO_POINTER(1));
  adw_header_bar_pack_start(ADW_HEADER_BAR(header), horizontal);
  adw_header_bar_pack_start(ADW_HEADER_BAR(header), vertical);
  GtkWidget* apply = gtk_button_new_with_label("Aplicar");
  gtk_widget_add_css_class(apply, "suggested-action");
  adw_header_bar_pack_end(ADW_HEADER_BAR(header), apply);
  adw_toolbar_view_add_top_bar(ADW_TOOLBAR_VIEW(toolbar), header);
  adw_toolbar_view_set_content(ADW_TOOLBAR_VIEW(toolbar), page);
  adw_dialog_set_child(dialog->dialog, toolbar);

  g_signal_connect(apply, "clicked", G_CALLBACK(apply_transform), dialog);
  g_signal_connect(horizontal, "clicked", G_CALLBACK(flip_transform), dialog);
  g_signal_connect(vertical, "clicked", G_CALLBACK(flip_transform), dialog);
  g_signal_connect(window, "closed", G_CALLBACK(+[](AdwDialog*, gpointer data) {
                     delete static_cast<TransformDialog*>(data);
                   }),
                   dialog);

  adw_dialog_present(dialog->dialog, host);
}

}  // namespace lienzo::gnome
