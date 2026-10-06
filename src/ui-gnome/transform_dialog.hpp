#pragma once

#include "ui-gnome/canvas.hpp"

#include <gtk/gtk.h>

namespace lienzo::gnome {

void present_layer_transform(
    GtkWidget* host,
    const CanvasView& canvas);

}  // namespace lienzo::gnome
