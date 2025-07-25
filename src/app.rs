use std::{cell::RefCell, rc::Rc};

use crate::generate::generate;
use crate::point::{interpolate, Point};
use egui::{pos2, Color32, FontDefinitions, FontFamily, Pos2, Stroke, TextEdit, Vec2};
#[allow(deprecated)]
use egui_extras::RetainedImage;
use std::sync::Arc;

/*
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
#[allow(deprecated)]
pub struct PathyApp {
    /// Physical size
    pub size: f32,
    /// Screen scale
    pub scale: u32,
    /// Current cursor mode
    #[serde(skip)]
    pub cursor_mode: CursorMode,
    /// Current background image
    #[serde(skip)]
    pub overlay: Option<RetainedImage>,
    /// Uploaded background image data
    pub uploaded: Option<Arc<[u8]>>,
    /// Field background state
    pub background: Background,
    /// Bezier points
    #[serde(skip)]
    pub points: Vec<Rc<RefCell<Point>>>,
    /// Locked selected point
    #[serde(skip)]
    pub selected: Option<Rc<RefCell<Point>>>,
    /// Inspected point
    #[serde(skip)]
    pub inspecting: Option<Rc<RefCell<Point>>>,
    /// Generated code
    pub generated: String,
    /// Generated save data
    pub save_data: String,
}

impl Default for PathyApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            size: 140.5,
            scale: 720,
            cursor_mode: CursorMode::Default,
            overlay: None,
            uploaded: None,
            background: Background::Game,
            points: Vec::new(),
            selected: None,
            inspecting: None,
            generated: String::new(),
            save_data: String::new(),
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

        // load saved path
        if let Some(storage) = cc.storage {
            app.load_save(&eframe::get_value::<Vec<Point>>(storage, "path").unwrap_or_default());
        }

        // Generate code and load overlay on startup
        app.generate();
        app.load_field_overlay();
        app
    }
    /// Update generated code
    fn generate(&mut self) {
        self.generated = generate(&self.points);
    }
    /// Update field image
    #[allow(deprecated)]
    fn load_field_overlay(&mut self) {
        match self.background {
            Background::Game => {
                self.overlay = Some(
                    RetainedImage::from_image_bytes("", include_bytes!("../assets/pushback.png"))
                        .unwrap(),
                );
            }
            Background::Skills => {
                self.overlay = Some(
                    RetainedImage::from_image_bytes(
                        "",
                        include_bytes!("../assets/pushback-skills.png"),
                    )
                    .unwrap(),
                );
            }
            Background::Custom => {
                self.overlay = self
                    .uploaded
                    .as_ref()
                    .and_then(|bytes| RetainedImage::from_image_bytes("", bytes).ok());
            }
        }
    }
    /// Gets the Bezier points in their save state
    fn get_save(&self) -> Vec<Point> {
        self.points.iter().map(|p| p.borrow().clone()).collect()
    }
    /// Loads saved Bezier points
    fn load_save(&mut self, save: &Vec<Point>) {
        self.points = save
            .iter()
            .map(|p| Rc::new(RefCell::new(p.clone())))
            .collect();
    }
}

impl eframe::App for PathyApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // save app state
        eframe::set_value(storage, eframe::APP_KEY, self);
        eframe::set_value(storage, "path", &self.get_save());
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    #[allow(deprecated)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                ui.label("Pathy v2.1.0");
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

        egui::SidePanel::right("side").show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut updated = false;
                    if let Some(point_ref) = &self.inspecting.clone() {
                        let mut point = point_ref.borrow_mut();
                        ui.label("Point Inspector");
                        ui.separator();
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("X: ");
                                let x = point.x;
                                let mut text = format!("{x:.3}");
                                if ui.text_edit_singleline(&mut text).has_focus() {
                                    point.x = text.parse().unwrap_or(x);
                                    updated = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Y: ");
                                let y = point.y;
                                let mut text = format!("{y:.3}");
                                if ui.text_edit_singleline(&mut text).has_focus() {
                                    point.y = text.parse().unwrap_or(y);
                                    updated = true;
                                }
                            });
                        });
                    }
                    if updated {
                        self.generate();
                    }
                    ui.label("Code");
                    ui.separator();
                    ui.add(
                        TextEdit::multiline(&mut self.generated.clone())
                            .font(egui::FontId::monospace(12.0))
                            .desired_width(f32::INFINITY),
                    );
                    ui.label("Save Data");
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.save_data = serde_json::to_string(&self.get_save()).unwrap();
                        }
                        if ui.button("Load").clicked() {
                            if let Ok(data) = serde_json::from_str::<Vec<Point>>(&self.save_data) {
                                self.load_save(&data);
                                self.generate();
                            }
                        }
                    });
                    ui.add(
                        TextEdit::multiline(&mut self.save_data)
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
                        self.uploaded = file.clone().bytes;
                        self.load_field_overlay();
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
            if self.points.len() >= 2 {
                for idx in 0..self.points.len() - 1 {
                    let a = self.points[idx].borrow_mut();
                    let mut b = self.points[idx + 1].borrow_mut();
                    let steps =
                        f32::floor(Pos2::from(b.clone()).distance(Pos2::from(a.clone()))) as usize;
                    // evaluate each pair
                    let draw_steps = if !b.animated {
                        ctx.animate_value_with_time(ui.make_persistent_id(b.id), steps as f32, 0.15)
                            as usize
                    } else {
                        steps
                    };
                    // Lock once animation completed
                    // So step size changes don't animate
                    if draw_steps >= steps {
                        b.animated = true;
                    }
                    for i in 1..draw_steps {
                        let point: Pos2 = interpolate(
                            &a.screen_clone(self.scale as f32 / self.size, rect.min),
                            &b.screen_clone(self.scale as f32 / self.size, rect.min),
                            i as f32 / steps as f32,
                        )
                        .into();
                        ui.painter().circle_filled(point, 2.0, Color32::YELLOW);
                        // If insert mode, find closest point
                        if self.cursor_mode == CursorMode::Insert {
                            if let Some(pos) = resp.hover_pos() {
                                let dist = point.distance_sq(pos);
                                if dist < min_dis {
                                    min_dis = dist;
                                    closest = Some(point);
                                    closest_idx = idx;
                                }
                            }
                        }
                    }
                }
            }

            // Draw points & check for selection
            let mut selected: Option<Rc<RefCell<Point>>> = None; // references currently selected point
            let mut idx: Option<usize> = None;
            for (i, point) in &mut self.points.iter_mut().enumerate() {
                let hovered = point.borrow_mut().draw(
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
                if hovered {
                    idx = Some(i);
                    selected = Some(point.clone());
                }
            }
            if let Some(point) = &selected {
                self.inspecting = Some(point.clone());
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
                            self.points.push(Rc::new(RefCell::new(Point::new(x, y))));
                            if !self.points.is_empty() {
                                // setup initial animation value
                                ctx.animate_value_with_time(
                                    ui.make_persistent_id(self.points.last().unwrap().borrow().id),
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
                            self.points
                                .insert(closest_idx + 1, Rc::new(RefCell::new(Point::new(x, y))));
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
                    ui.label("Drag an drop an image to set the field background!");
                }
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
