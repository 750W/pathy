/*
 * PATHY - A tool for creating complex paths
 * in autonomous code without having to manually write
 * each and every variable.
 *
 * Created by Daksh Gupta.
 */
pub use crate::app::PathyApp;
use crate::app::*;
use eframe::egui::Visuals;
use egui::{pos2, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use egui_extras::RetainedImage;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.

impl eframe::App for PathyApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Dark mode
        ctx.set_visuals(Visuals::dark());
        let Self {
            height,
            width,
            scale,
            mode,
            path,
            selected,
            overlay,
            result,
            processed,
        } = self;

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Settings");

            // Size settings
            let response = ui.add_enabled_ui(path.is_empty(), |ui| {
                ui.label("Field Dimensions:");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(height));
                    ui.label("Height (Inches)")
                });
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(width));
                    ui.label("Width (Inches)")
                });
                ui.add(egui::Slider::new(scale, 0.0..=1000.0).text("Scale"));
            });
            if !path.is_empty() {
                response.response.on_hover_text_at_pointer(
                    "Size settings may not be changed once you've created a path.",
                );
            }
            ui.label("Drop an image to set an overlay!");

            // Notice
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Made for ");
                    ui.hyperlink_to("EZTemplate", "https://ez-robotics.github.io/EZ-Template/");
                    ui.label(" and ");
                    ui.hyperlink_to("PROS", "https://pros.cs.purdue.edu");
                    ui.label(".");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("Pathy");
            ui.label("Created by Daksh Gupta.");
            ui.label("Made for use in auton code. Generated code uses the EZTemplate API.");
            egui::warn_if_debug_build(ui);
        });

        // Path Designer
        egui::Window::new("Path Designer").show(ctx, |ui| {
            // If the width is scale, find the height that keeps it
            // in the correct aspect ratio
            let aspecty: f32 = (*height / *width) * *scale;
            ui.heading("Path Designer");
            // UI Buttons
            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    *mode = CursorMode::Create;
                    // Reset processes
                    *processed = Vec::new();
                }
                if ui.button("Edit").clicked() {
                    *mode = CursorMode::Edit;
                    *processed = Vec::new();
                }
                if ui.button("Delete").clicked() {
                    *mode = CursorMode::Delete;
                    *processed = Vec::new();
                }
                if ui.button("Trim").clicked() {
                    *mode = CursorMode::Trim;
                    *processed = Vec::new();
                }
                if ui.button("Clear").clicked() {
                    *path = Vec::new(); // Clear path
                    *processed = Vec::new();
                    *result = None;
                }
                // Both an "optimization" and a hacky workaround (see https://github.com/750W/pathy/issues/1)
                if processed.is_empty() && ui.button("Preprocess").clicked() {
                    *processed = Self::preprocess(path, (*scale, aspecty), (*width, *height));
                }
                // Order here is important, as the ui button is only rendered if the first
                // condition is true. Otherwise, there's no point in evaluating the second
                // condition, thus not rendering the button.
                if !processed.is_empty() && ui.button("Generate").clicked() {
                    *result = Some(Self::generate(processed));
                }
                if *mode != CursorMode::Default && ui.button("Finish").clicked() {
                    *mode = CursorMode::Default;
                }
            });
            ui.add_space(10.0);
            // Render the bounds
            let (rect, response) = ui.allocate_at_least(
                Vec2 {
                    x: *scale,
                    y: aspecty,
                },
                Sense::click_and_drag(),
            );
            ui.painter().rect(
                rect,
                0.0,
                Color32::from_gray(64),
                Stroke::new(5.0, Color32::WHITE),
            );
            // Check for dropped files
            ctx.input(|i| {
                if let Some(file) = i.raw.dropped_files.last() {
                    if let Some(bytes) = file.clone().bytes {
                        if let Ok(image) = RetainedImage::from_image_bytes("", &bytes) {
                            *overlay = Some(image);
                        }
                    }
                }
            });
            // Render image
            match overlay {
                Some(image) => {
                    //image.show_scaled(ui, *scale / (image.width() as f32));
                    ui.painter().image(
                        image.texture_id(ctx),
                        rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                None => (),
            }
            // Variable we'll use to render lines
            let mut prev: Option<Pos2> = None;
            // Line stroke
            let yellow_line = Stroke {
                width: 3.0,
                color: Color32::YELLOW,
            };
            // Render all lines
            path.iter().for_each(|pos| {
                let screen_pos = Pos2 {
                    x: response.rect.min.x + pos.x,
                    y: response.rect.min.y + pos.y,
                };
                // Render lines
                if let Some(prev_pos) = prev {
                    ui.painter()
                        .line_segment([prev_pos, screen_pos], yellow_line);
                };
                prev = Some(screen_pos);
            });
            // Render all points
            path.iter().enumerate().for_each(|(idx, pos)| {
                let screen_pos = Pos2 {
                    x: response.rect.min.x + pos.x,
                    y: response.rect.min.y + pos.y,
                };
                // Render points
                if idx == 0 {
                    ui.painter().circle_filled(screen_pos, 5.0, Color32::GREEN);
                } else if idx == path.len() - 1 {
                    ui.painter().circle_filled(screen_pos, 5.0, Color32::BLUE);
                } else {
                    ui.painter().circle_filled(screen_pos, 5.0, Color32::YELLOW);
                }
            });
            // Hovered point, Edit and Delete will actually set this to be the closest point.
            let mut hovered = response.hover_pos();
            // Index of selected point of path (used by edit & delete)
            let mut sl_idx: Option<usize> = None;
            // Render the tooltips
            match *mode {
                CursorMode::Default => (), // No tooltip
                CursorMode::Create => {
                    // Get pointer position
                    if let Some(pos) = hovered {
                        // Put circle under cursor
                        ui.painter().circle_filled(pos, 5.0, Color32::YELLOW);
                        // Render line preview
                        if let Some(prev_pos) = prev {
                            ui.painter().line_segment([prev_pos, pos], yellow_line);
                        }
                    }
                }
                CursorMode::Edit | CursorMode::Delete | CursorMode::Trim => {
                    // Get pointer position
                    if let Some(hover_pos) = hovered {
                        // Find the nearest point. We just add the x and y differences without
                        // squaring them, since we don't need the actual distance, just
                        // something we can compare (and works 99% of the time).
                        let mut distance = f32::MAX;
                        hovered = Some(path.iter().enumerate().fold(
                            hover_pos,
                            |old_pos, (idx, pos)| {
                                let screen_pos = Pos2 {
                                    x: response.rect.min.x + pos.x,
                                    y: response.rect.min.y + pos.y,
                                };
                                let dis = f32::abs(hover_pos.x - screen_pos.x)
                                    + f32::abs(hover_pos.y - screen_pos.y);
                                if dis < distance {
                                    distance = dis;
                                    sl_idx = Some(idx);
                                    return screen_pos;
                                }
                                old_pos
                            },
                        ));
                        // Render closest point red
                        ui.painter()
                            .circle_filled(hovered.unwrap(), 5.1, Color32::RED);
                    }
                }
            }
            // Handle clicks
            // I'd like to do this first, so there's no frame delay, but it's a little more
            // idiomatic for me to do it this way, since we can now use a match statement above
            // (since Delete mode uses the nearest point calculated for the tooltip).
            if response.clicked() {
                match mode {
                    // Default does nothing, and Edit uses drags
                    CursorMode::Default | CursorMode::Edit => (),
                    // Add cursor position to list
                    CursorMode::Create => {
                        if let Some(pos) = ctx.pointer_interact_pos() {
                            path.push(Pos2 {
                                x: pos.x - response.rect.min.x,
                                y: pos.y - response.rect.min.y,
                            });
                        }
                    }
                    // Delete cursor position (slices vector)
                    CursorMode::Delete => {
                        if let Some(idx) = sl_idx {
                            path.remove(idx); // Deletes the elements
                        }
                    }
                    CursorMode::Trim => {
                        if let Some(idx) = sl_idx {
                            path.drain(idx..); // Deletes elements from idx
                        }
                    }
                }
            }
            // Handle drags - Edit mode only
            if *mode == CursorMode::Edit && !path.is_empty() {
                // Set selected at drag start
                if response.drag_started() {
                    // Drag started, set current index as selected.
                    // This is to prevent, say, dragging over another point from stealing focus from
                    // the currently selected point.
                    if let Some(idx) = sl_idx {
                        *selected = idx;
                    }
                }
                // Move the selected point
                if response.dragged() {
                    if let Some(pos) = ctx.pointer_interact_pos() {
                        path[*selected] = Pos2 {
                            x: pos.x - response.rect.min.x,
                            y: pos.y - response.rect.min.y,
                        }
                    }
                }
            }
        });

        match result {
            Some(code) => {
                egui::Window::new("Generated Code").show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.code_editor(code);
                    });
                });
            }
            None => (),
        }
    }
}
