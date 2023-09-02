use eframe::egui::Visuals;
use egui::{Color32, Pos2, Sense, Stroke, Vec2};
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // this how you opt-out of serialization of a member
    //#[serde(skip)]
    height: f32,      // Height of field
    width: f32,       // Width of field
    scale: f32,       // Scale to display
    mode: CursorMode, // Cursor mode
    path: Vec<Pos2>,  // Current path
    selected: usize,  // Current selected node (edit mode)
}

#[derive(serde::Deserialize, serde::Serialize, Eq, PartialEq)]
enum CursorMode {
    // Represent possible cursor modes
    Default, // No action
    Create,  // Create new nodes and paths
    Edit,    // Edit the positioning of points
    Delete,  // Delete points (whilst still keeping only one path).
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            height: 1776.9,
            width: 1475.3,
            scale: 600.0,
            mode: CursorMode::Default,
            path: Vec::new(),
            selected: 0,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
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
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

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

            let response = ui.add_enabled_ui(path.len() == 0, |ui| {
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
            if path.len() != 0 {
                response.response.on_hover_text_at_pointer(
                    "Size settings may not be changed once you've created a path.",
                );
            }

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
                if ui.button("Save").clicked() {
                    *mode = CursorMode::Default;
                    println!("Not yet implemented")
                }
                if ui.button("Create").clicked() {
                    *mode = CursorMode::Create;
                }
                if ui.button("Edit").clicked() {
                    *mode = CursorMode::Edit;
                }
                if ui.button("Delete").clicked() {
                    *mode = CursorMode::Delete;
                }
                if ui.button("Clear").clicked() {
                    *path = Vec::new(); // Clear path
                }
                // Order here is important, as the ui button is only rendered if the first
                // condition is true. Otherwise, there's no point in evaluating the second
                // condition, thus not rendering the button.
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
            // Variable we'll use to render lines
            let mut prev: Option<Pos2> = None;
            // Line stroke
            let yellow_line = Stroke {
                width: 3.0,
                color: Color32::YELLOW,
            };
            // Render all points and lines
            path.iter().enumerate().for_each(|(idx, pos)| {
                let screen_pos = Pos2 {
                    x: response.rect.min.x + pos.x,
                    y: response.rect.min.y + pos.y,
                };
                // Render lines
                match prev {
                    Some(prev_pos) => {
                        ui.painter()
                            .line_segment([prev_pos, screen_pos], yellow_line);
                    }
                    None => (),
                };
                prev = Some(screen_pos);
                // Render points
                if idx == 0 {
                    ui.painter().circle_filled(screen_pos, 5.0, Color32::GREEN);
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
                    match hovered {
                        // Put circle under cursor
                        Some(pos) => {
                            ui.painter().circle_filled(pos, 5.0, Color32::YELLOW);
                            // Render line preview
                            match prev {
                                Some(prev_pos) => {
                                    ui.painter().line_segment([prev_pos, pos], yellow_line)
                                }
                                None => (),
                            }
                        }
                        None => (),
                    }
                }
                CursorMode::Edit | CursorMode::Delete => {
                    // Get pointer position
                    match hovered {
                        Some(hover_pos) => {
                            // Find the nearest point, using weird but generally more effecient
                            // algorithm.
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
                        None => (),
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
                    CursorMode::Create => match ctx.pointer_interact_pos() {
                        Some(pos) => path.push(Pos2 {
                            x: pos.x - response.rect.min.x,
                            y: pos.y - response.rect.min.y,
                        }),
                        None => (),
                    },
                    // Delete cursor position (slices vector)
                    CursorMode::Delete => match sl_idx {
                        Some(idx) => drop(path.drain(idx..)), // Deletes the elements
                        None => (),
                    },
                }
            }
            // Handle drags - Edit mode only
            if *mode == CursorMode::Edit {
                // Set selected at drag start
                if response.drag_started() {
                    // Drag started, set current index as selected.
                    // This is to prevent, say, dragging over another point from stealing focus from
                    // the currently selected point.
                    match sl_idx {
                        Some(idx) => *selected = idx,
                        None => (),
                    }
                }
                // Move the selected point
                if response.dragged() {
                    match ctx.pointer_interact_pos() {
                        Some(pos) => {
                            path[*selected] = Pos2 {
                                x: pos.x - response.rect.min.x,
                                y: pos.y - response.rect.min.y,
                            }
                        }
                        None => (),
                    }
                }
            }
        });
    }
}
