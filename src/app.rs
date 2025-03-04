use std::{cell::RefCell, rc::Rc};

use crate::bezier::{interpolate, interpolate_slope, BezPoint, Point};
use crate::generate::generate;
use egui::{pos2, Color32, FontDefinitions, FontFamily, FontId, Pos2, Stroke, TextEdit, Vec2};
use egui_extras::RetainedImage;

// Uncomment this section to get access to the console_log macro
// Use console_log to print things to console. println macro doesn't work
// here, so you'll need it.
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// */
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CursorMode {
    Default,
    Create,
    Insert,
    Delete,
    Trim,
}

/// Represents chosen background image.
#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
pub enum Background {
    Game,
    Skills,
    Custom,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PathyApp {
    /// Physical size
    pub size: f32,
    /// Screen scale
    pub scale: u32,
    /// Current cursor mode
    #[serde(skip)]
    pub cursor_mode: CursorMode,
    /// Uploaded background image
    #[serde(skip)]
    pub overlay: Option<RetainedImage>,
    /// Field background image
    pub background: Background,
    /// Bezier points
    pub points: Vec<BezPoint>,
    /// Number of steps
    pub steps: usize,
    /// Locked selected point
    #[serde(skip)]
    pub selected: Option<Rc<RefCell<Point>>>,
    /// Generated code
    pub generated: String,
}

impl Default for PathyApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            size: 140.5,
            scale: 720,
            cursor_mode: CursorMode::Default,
            overlay: None,
            background: Background::Game,
            points: Vec::new(),
            steps: 100,
            selected: None,
            generated: String::new(),
        }
    }
}

impl PathyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "SpaceGrotesk".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/SpaceGrotesk-Regular.ttf"
            ))),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "SpaceGrotesk".into());
        cc.egui_ctx.set_fonts(fonts);
        // only if in dark mode
        cc.egui_ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.visuals.panel_fill = Color32::from_gray(10);
        });

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let mut app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        // Generate code and load overlay on startup
        app.generate();
        app.load_field_overlay();
        app
    }
    /// Update generated code
    fn generate(&mut self) {
        self.generated = generate(&self.points, 0.1);
    }
    /// Update field image
    fn load_field_overlay(&mut self) {
        match self.background {
            Background::Game => {
                self.overlay = Some(
                    RetainedImage::from_image_bytes(
                        "",
                        include_bytes!("../assets/high-stakes-game.png"),
                    )
                    .unwrap(),
                );
            }
            Background::Skills => {
                self.overlay = Some(
                    RetainedImage::from_image_bytes(
                        "",
                        include_bytes!("../assets/high-stakes-skills.png"),
                    )
                    .unwrap(),
                );
            }
            Background::Custom => {
                self.overlay = None;
            }
        }
    }
}

impl eframe::App for PathyApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                ui.label("Pathy v2.0.0");
                ui.separator();
                ui.label("Field Size: ");
                ui.add_enabled_ui(self.points.is_empty(), |ui| {
                    ui.add(egui::DragValue::new(&mut self.size).suffix(" inches"));
                })
                .response
                .on_disabled_hover_text("Field size may not be changed once path is created.");
                ui.label("Field Scale: ");
                ui.add(
                    egui::DragValue::new(&mut self.scale)
                        .suffix("px")
                        .speed(2.5),
                )
                .on_hover_text("Screen scale of the field");
                ui.label("Point Density: ");
                ui.add(egui::DragValue::new(&mut self.steps).speed(2.5))
                    .on_hover_text("Number of points to display for each curve");
                ui.separator();
                /* BUTTON LOGIC */
                let modes = [
                    (egui::Key::C, CursorMode::Create, "Create new point"),
                    (egui::Key::I, CursorMode::Insert, "Insert point in path"),
                    (egui::Key::D, CursorMode::Delete, "Delete a single point"),
                    (egui::Key::T, CursorMode::Trim, "Trim path to point"),
                ];
                // Custom selectable label lets us double click to return to default
                for (key, mode, desc) in modes {
                    if ui
                        .add(egui::SelectableLabel::new(
                            self.cursor_mode == mode,
                            format!("{mode:?}"), // since we derive debug
                        ))
                        .on_hover_text(format!("{desc} ({})", format!("{:?}", key).to_lowercase()))
                        .clicked()
                    {
                        if self.cursor_mode != mode {
                            self.cursor_mode = mode.clone();
                        } else {
                            self.cursor_mode = CursorMode::Default;
                        }
                    }
                    // also check key press
                    ctx.input(|input| {
                        if input.key_pressed(key) {
                            if self.cursor_mode != mode {
                                self.cursor_mode = mode;
                            } else {
                                self.cursor_mode = CursorMode::Default;
                            }
                        }
                    });
                }
                ui.separator();
                if ui
                    .button("Generate")
                    .on_hover_text("Generate path code")
                    .clicked()
                {
                    self.generate();
                };
                if ui.button("Clear").on_hover_text("Clear path").clicked() {
                    self.points.clear();
                    self.generate();
                };
                ui.separator();
                ui.label("Field: ");
                // store functions to lazily load images
                // do NOT load these every frame
                let fields = [Background::Game, Background::Skills, Background::Custom];
                for field in fields {
                    if ui
                        .add(egui::SelectableLabel::new(
                            self.background == field,
                            format!("{field:?}"),
                        ))
                        .clicked()
                    {
                        self.background = field;
                        self.load_field_overlay();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                    ui.separator();
                });
            });
        });

        egui::SidePanel::right("Code").show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label("Code");
                    ui.separator();
                    ui.add(
                        TextEdit::multiline(&mut self.generated.clone())
                            .font(egui::FontId::monospace(12.0))
                            .desired_width(f32::INFINITY),
                    );
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            /* FIELD RENDERING */
            let (rect, resp) = ui.allocate_exact_size(
                Vec2 {
                    x: self.scale as f32,
                    y: self.scale as f32,
                },
                egui::Sense::click_and_drag(),
            );
            // Check for dropped image
            if self.background == Background::Custom {
                ctx.input(|i| {
                    if let Some(file) = i.raw.dropped_files.last() {
                        if let Some(bytes) = file.clone().bytes {
                            if let Ok(image) = RetainedImage::from_image_bytes("", &bytes) {
                                self.overlay = Some(image);
                            }
                        }
                    }
                });
            }
            match &self.overlay {
                Some(image) => {
                    ui.painter().image(
                        image.texture_id(ctx),
                        rect,
                        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                _ => {
                    ui.painter().rect(
                        rect,
                        0.0,
                        match ctx.theme() {
                            egui::Theme::Dark => Color32::from_gray(30),
                            egui::Theme::Light => Color32::from_gray(180),
                        },
                        Stroke::NONE,
                    );
                }
            }

            /* POINT RENDERING + HOVER DETECTION */
            // Render curve points
            let mut min_dis = f32::MAX;
            let mut closest: Option<Pos2> = None;
            let mut closest_idx: usize = 0;
            let mut closest_step: f32 = 0.0;
            if self.points.len() >= 2 {
                for idx in 0..self.points.len() - 1 {
                    let a = self.points[idx].clone();
                    let b = self.points[idx + 1].clone();
                    // evaluate each pair
                    let draw_steps = if !b.animated {
                        ctx.animate_value_with_time(
                            ui.make_persistent_id(b.id),
                            self.steps as f32,
                            0.3,
                        ) as usize
                    } else {
                        self.steps
                    };
                    // Lock once animation completed
                    // So step size changes don't animate
                    if draw_steps >= self.steps {
                        self.points[idx + 1].animated = true;
                    }
                    for i in 1..draw_steps {
                        let point = interpolate(&a, &b, i as f32 / self.steps as f32)
                            .screen(self.scale as f32 / self.size, rect.min);
                        ui.painter().circle_filled(point, 2.0, Color32::YELLOW);
                        // If insert mode, find closest point
                        if self.cursor_mode == CursorMode::Insert {
                            if let Some(pos) = resp.hover_pos() {
                                let dist = point.distance_sq(pos);
                                if dist < min_dis {
                                    min_dis = dist;
                                    closest = Some(point);
                                    closest_idx = idx;
                                    closest_step = i as f32 / self.steps as f32;
                                }
                            }
                        }
                    }
                }
            }

            let mut selected: Option<Rc<RefCell<Point>>> = None; // references currently selected point
            let mut idx: Option<usize> = None;
            for (i, point) in &mut self.points.iter_mut().enumerate() {
                let res = point.draw(
                    ui,
                    ctx,
                    self.scale as f32 / self.size,
                    rect.min,
                    if self.cursor_mode == CursorMode::Trim {
                        if idx.is_some() {
                            &CursorMode::Trim
                        } else {
                            &CursorMode::Delete
                        }
                    } else {
                        &self.cursor_mode
                    },
                    if selected.is_none() {
                        resp.hover_pos()
                    } else {
                        None
                    }, // ensure only 1 point gets selected
                );
                idx = idx.or(if res.is_some() { Some(i) } else { None });
                selected = selected.or(res);
            }

            /* INPUT HANDLERS */
            if ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary))
                && !matches!(self.cursor_mode, CursorMode::Delete | CursorMode::Trim)
            {
                // Lock selection in case of drag
                if self.selected.is_none() {
                    if let Some(point) = &selected {
                        point.borrow_mut().locked = true;
                        self.selected = Some(point.clone());
                    }
                }
            }
            if ctx.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
                // Unlock any selection
                if let Some(point) = &self.selected {
                    point.borrow_mut().locked = false;
                    self.selected = None;
                }
            }
            if resp.clicked() {
                match &self.cursor_mode {
                    CursorMode::Create => {
                        if selected.is_some() {
                            return;
                        }
                        if let Some(pos) = resp.hover_pos() {
                            // Ensure points within bounds
                            if pos.x < rect.min.x
                                || pos.x > rect.width() + rect.min.x
                                || pos.y < rect.min.y
                                || pos.y > rect.height() + rect.min.y
                            {
                                return;
                            }
                            // Calculate points relative to field
                            let x = (pos.x - rect.min.x) * (self.size / self.scale as f32);
                            let y = (pos.y - rect.min.y) * (self.size / self.scale as f32);
                            console_log!("({}, {})", x, y);
                            if self.points.is_empty() {
                                self.points
                                    .push(BezPoint::new(x, y, x - 10.0, y, x + 10.0, y));
                            } else {
                                let Pos2 { x: ix, y: iy } =
                                    Pos2::from(self.points.last().unwrap().cp2.borrow().clone())
                                        .lerp(pos2(x, y), 0.5);
                                self.points.push(BezPoint::new(
                                    x,
                                    y,
                                    ix,
                                    iy,
                                    2.0 * x - ix,
                                    2.0 * y - iy,
                                ));
                                // setup initial animation value
                                ctx.animate_value_with_time(
                                    ui.make_persistent_id(self.points.last().unwrap().id),
                                    0.0,
                                    0.5,
                                );
                            }
                            self.generate();
                        }
                    }
                    CursorMode::Delete => {
                        if let Some(i) = idx {
                            self.points.remove(i);
                            self.generate();
                        }
                    }
                    CursorMode::Trim => {
                        if let Some(i) = idx {
                            self.points.truncate(i);
                            self.generate();
                        }
                    }
                    CursorMode::Insert => {
                        if let Some(pos) = closest {
                            let x = (pos.x - rect.min.x) * (self.size / self.scale as f32);
                            let y = (pos.y - rect.min.y) * (self.size / self.scale as f32);
                            // Calculate future x and ys
                            let Point { x: fx, y: fy, .. } = interpolate(
                                &self.points[closest_idx],
                                &self.points[closest_idx + 1],
                                closest_step + 0.1,
                            );
                            self.points.insert(
                                closest_idx + 1,
                                BezPoint::new(x, y, 2.0 * x - fx, 2.0 * y - fy, fx, fy),
                            );
                            self.generate();
                        }
                    }
                    _ => {}
                }
            }

            if resp.dragged() && resp.contains_pointer() {
                let mut changed = false;
                if let Some(point) = &self.selected {
                    if let Some(pos) = ctx.pointer_interact_pos() {
                        if let Ok(mut p) = point.try_borrow_mut() {
                            p.x = (pos.x - rect.min.x) * (self.size / self.scale as f32);
                            p.y = (pos.y - rect.min.y) * (self.size / self.scale as f32);
                            changed = true;
                        } else {
                            console_log!("ERROR: Failed to update point!");
                        }
                    }
                }
                if changed {
                    self.generate(); // afterwards to please borrow checker
                }
            }

            /* TOOLTIPS */
            match &self.cursor_mode {
                CursorMode::Create => {
                    // Display circle under pointer
                    if self.selected.is_some() || selected.is_some() {
                        return;
                    }
                    if let Some(pos) = resp.hover_pos() {
                        ui.painter()
                            .circle_stroke(pos, 5.0, Stroke::new(2.0, Color32::YELLOW));
                    }
                }
                CursorMode::Insert => {
                    // Display circle under closest point
                    if let Some(pos) = closest {
                        ui.painter()
                            .circle_stroke(pos, 5.0, Stroke::new(2.0, Color32::YELLOW));
                    }
                }
                _ => {}
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                if self.background == Background::Custom && self.overlay.is_none() {
                    ui.label("Drop image here to set custom background");
                }
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
