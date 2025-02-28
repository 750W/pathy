use egui::{lerp, pos2, Color32, Context, Pos2, Stroke, Ui, Vec2};
use egui_extras::RetainedImage;
use uuid::Uuid;

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
#[derive(PartialEq, Eq)]
pub enum CursorMode {
    Default,
    Create,
    Edit,
    Delete,
}

/// A Bezier point.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct BezPoint {
    pub x: f32,
    pub y: f32,
    pub cp1x: f32,
    pub cp1y: f32,
    pub cp2x: f32,
    pub cp2y: f32,

    // ID
    id: Uuid,
    // Hover states
    p_hov: bool,
    cp1_hov: bool,
    cp2_hov: bool,
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
    /// Background image
    #[serde(skip)]
    pub overlay: Option<RetainedImage>,
    /// Bezier points
    pub points: Vec<BezPoint>,
}

impl BezPoint {
    fn new(x: f32, y: f32, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32) -> Self {
        Self {
            x,
            y,
            cp1x,
            cp1y,
            cp2x,
            cp2y,
            id: Uuid::new_v4(),
            p_hov: false,
            cp1_hov: false,
            cp2_hov: false,
        }
    }
    fn draw(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        ratio: f32,
        origin: Pos2,
        hover_pos: Option<Pos2>,
    ) {
        let r = 5.0; // point radius
        let r_hov = 8.0; // hover radius

        // Generate an id to keep track of point animations
        let id = ui.make_persistent_id(self.id);
        let p_id = id.with(0);
        let cp1_id = id.with(1);
        let cp2_id = id.with(2);

        // Main point
        let x = self.x * ratio + origin.x;
        let y = self.y * ratio + origin.y;
        // Control points
        let cp1x = self.cp1x * ratio + origin.x;
        let cp1y = self.cp1y * ratio + origin.y;
        let cp2x = self.cp2x * ratio + origin.x;
        let cp2y = self.cp2y * ratio + origin.y;

        // Update hover states
        self.p_hov = false;
        self.cp1_hov = false;
        self.cp2_hov = false;
        if let Some(hover_pos) = hover_pos {
            let mut distances = [
                (&mut self.cp1_hov, hover_pos.distance_sq(pos2(cp1x, cp1y))),
                (&mut self.cp2_hov, hover_pos.distance_sq(pos2(cp2x, cp2y))),
            ];
            let mut point_dis = (&mut self.p_hov, hover_pos.distance_sq(pos2(x, y)));
            let (hovered, min_distance) =
                distances
                    .iter_mut()
                    .fold(&mut point_dis, |min, x| if x.1 < min.1 { x } else { min });
            if *min_distance < r * r {
                console_log!("Hovered!");
                **hovered = true;
                console_log!(
                    "POINT: {}\nCONTROL POINT 1: {}\nCONTROL POINT 2: {}",
                    self.p_hov,
                    self.cp1_hov,
                    self.cp2_hov
                );
            } else {
                **hovered = false;
            }
        }

        // Update point radii based on hover state
        let p_r = lerp(r..=r_hov, ctx.animate_bool(p_id, self.p_hov));
        let cp1_r = lerp(r..=r_hov, ctx.animate_bool(cp1_id, self.cp1_hov));
        let cp2_r = lerp(r..=r_hov, ctx.animate_bool(cp2_id, self.cp2_hov));

        // Offsets to prevent lines draw inside hollow control points
        // Essentially just sine and cosine of the angle
        let dx1 = cp1x - x;
        let dy1 = cp1y - y;
        let dx2 = cp2x - x;
        let dy2 = cp2y - y;
        let mag1 = (dx1 * dx1 + dy1 * dy1).sqrt();
        let mag2 = (dx2 * dx2 + dy2 * dy2).sqrt();
        let xoffset1 = (cp1_r + 1.0) * dx1 / mag1;
        let yoffset1 = (cp1_r + 1.0) * dy1 / mag1;
        let xoffset2 = (cp2_r + 1.0) * dx2 / mag2;
        let yoffset2 = (cp2_r + 1.0) * dy2 / mag2;

        // Control lines
        ui.painter().line_segment(
            [pos2(x, y), pos2(cp1x - xoffset1, cp1y - yoffset1)],
            Stroke::new(2.0, Color32::YELLOW),
        );
        ui.painter().line_segment(
            [pos2(x, y), pos2(cp2x - xoffset2, cp2y - yoffset2)],
            Stroke::new(2.0, Color32::YELLOW),
        );

        // Draw points
        ui.painter().circle_filled(pos2(x, y), p_r, Color32::YELLOW);
        ui.painter()
            .circle_stroke(pos2(cp1x, cp1y), cp1_r, Stroke::new(2.0, Color32::YELLOW));
        ui.painter()
            .circle_stroke(pos2(cp2x, cp2y), cp2_r, Stroke::new(2.0, Color32::YELLOW));
    }
}

impl Default for PathyApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            size: 140.5,
            scale: 720,
            cursor_mode: CursorMode::Default,
            overlay: None,
            points: Vec::new(),
        }
    }
}

impl PathyApp {
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
                });
                ui.label("Field Scale: ");
                ui.add(
                    egui::DragValue::new(&mut self.scale)
                        .suffix("px")
                        .speed(2.5),
                );
                ui.separator();
                /* BUTTON LOGIC */
                let modes = [
                    (CursorMode::Create, "Create"),
                    (CursorMode::Edit, "Edit"),
                    (CursorMode::Delete, "Delete"),
                ];
                // Custom selectable label lets us double click to return to default
                for (mode, label) in modes {
                    if ui
                        .add(egui::SelectableLabel::new(self.cursor_mode == mode, label))
                        .clicked()
                    {
                        if self.cursor_mode != mode {
                            self.cursor_mode = mode;
                        } else {
                            self.cursor_mode = CursorMode::Default;
                        }
                    }
                }
                ui.separator();
                if ui.button("Generate").clicked() {
                    // TODO: generate logic
                    self.cursor_mode = CursorMode::Default;
                };
                if ui.button("Clear").clicked() {
                    self.points.clear();
                };
                ui.separator();
                if let None = self.overlay {
                    ui.label("Drop an image to set the field background!");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                    ui.separator();
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
            ctx.input(|i| {
                if let Some(file) = i.raw.dropped_files.last() {
                    if let Some(bytes) = file.clone().bytes {
                        if let Ok(image) = RetainedImage::from_image_bytes("", &bytes) {
                            self.overlay = Some(image);
                        }
                    }
                }
            });
            // Draw field background
            match &self.overlay {
                Some(image) => {
                    ui.painter().image(
                        image.texture_id(ctx),
                        rect,
                        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                None => {
                    ui.painter()
                        .rect(rect, 0.0, Color32::from_gray(64), Stroke::NONE);
                }
            }

            /* TOOLTIPS */
            match &self.cursor_mode {
                CursorMode::Create => {
                    // Display circle under pointer
                    if let Some(pos) = resp.hover_pos() {
                        ui.painter().circle(
                            pos,
                            5.0,
                            Color32::from_black_alpha(0),
                            Stroke::new(2.0, Color32::YELLOW),
                        );
                    }
                }
                CursorMode::Edit => {
                    // TODO
                }
                CursorMode::Delete => {
                    // TODO
                }
                _ => {}
            }

            /* POINT RENDERING */
            for point in &mut self.points {
                point.draw(
                    ui,
                    ctx,
                    self.scale as f32 / self.size,
                    rect.min,
                    resp.hover_pos(),
                );
            }

            /* CLICK HANDLERS */
            if resp.clicked() {
                match &self.cursor_mode {
                    CursorMode::Create => {
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
                            self.points
                                .push(BezPoint::new(x, y, x + 20.0, y + 10.0, x, y + 20.0));
                        }
                    }
                    CursorMode::Edit => {
                        // TODO
                    }
                    CursorMode::Delete => {
                        // TODO
                    }
                    _ => {}
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
