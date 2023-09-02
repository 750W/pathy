use eframe::egui::Visuals;
use egui::{Color32, Pos2, Sense, Stroke, Vec2};
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // this how you opt-out of serialization of a member
    //#[serde(skip)]
    height: f32,
    width: f32,
    scale: f32,
    mode: CursorMode,
    path: Vec<Pos2>,
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

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
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
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    *mode = CursorMode::Default;
                    println!("Not yet implemented")
                }
                if ui.button("Create Path").clicked() {
                    *mode = CursorMode::Create;
                }
                if ui.button("Edit Points").clicked() {
                    *mode = CursorMode::Edit;
                }
                if ui.button("Delete Points").clicked() {
                    *mode = CursorMode::Delete;
                }
                if ui.button("Clear Paths").clicked() {
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
                Sense::click(),
            );
            ui.painter().rect(
                rect,
                0.0,
                Color32::from_gray(64),
                Stroke::new(5.0, Color32::WHITE),
            );
            // Render the create tooltip
            // Might change to a case later
            if *mode == CursorMode::Create {
                // Get pointer position
                match ctx.pointer_hover_pos() {
                    // Put circle under cursor
                    Some(pos) => ui.painter().circle_filled(pos, 5.0, Color32::YELLOW),
                    None => (),
                }
            }
            if response.clicked() {
                match mode {
                    // Do nothing
                    CursorMode::Default => (),
                    // Add cursor position to list
                    CursorMode::Create => match ctx.pointer_interact_pos() {
                        Some(pos) => path.push(Pos2 {
                            x: pos.x - response.rect.min.x,
                            y: pos.y - response.rect.min.y,
                        }),
                        None => (),
                    },
                    CursorMode::Edit => (),
                    CursorMode::Delete => (),
                }
            }
            let mut prev: Option<Pos2> = None;
            // Render all points
            path.iter().for_each(|pos| {
                let abs_pos = Pos2 {
                    x: response.rect.min.x + pos.x,
                    y: response.rect.min.y + pos.y,
                };
                ui.painter().circle_filled(abs_pos, 5.0, Color32::YELLOW);
                match prev {
                    Some(prev_pos) => {
                        ui.painter().line_segment(
                            [prev_pos, abs_pos],
                            Stroke {
                                width: 3.0,
                                color: Color32::YELLOW,
                            },
                        );
                    }
                    None => (),
                };
                prev = Some(abs_pos);
            });
        });
    }
}
