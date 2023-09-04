/*
 * PATHY - A tool for creating complex paths
 * in autonomous code without having to manually write
 * each and every variable.
 *
 * Created by Daksh Gupta.
 */
use eframe::egui::Visuals;
use egui::{pos2, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use egui_extras::RetainedImage;
// Uncomment this section to get access to the console_log macro
// Use console_log to print things to console. println macro doesn't work
// here, so you'll need it.
/*use wasm_bindgen::prelude::*;
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
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PathyApp {
    // this how you opt-out of serialization of a member
    //#[serde(skip)]
    height: f32,             // Height of field
    width: f32,              // Width of field
    scale: f32,              // Scale to display
    mode: CursorMode,        // Cursor mode
    path: Vec<Pos2>,         // Current path
    selected: usize,         // Current selected node (edit mode)
    processed: Vec<Process>, // Processed fields
    #[serde(skip)] // We can't serialize and image; and we don't want to
    overlay: Option<RetainedImage>, // Uploaded overlay
    result: Option<String>,  // Final string
}

#[derive(serde::Deserialize, serde::Serialize, Eq, PartialEq)]
enum CursorMode {
    // Represent possible cursor modes
    Default, // No action
    Create,  // Create new nodes and paths
    Edit,    // Edit the positioning of points
    Delete,  // Delete points (whilst still keeping only one path).
    Trim,    // Trim points (whilst keeping one path)
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
enum Process {
    // Represents movements for the robot to take
    Drive(i32),
    Turn(i32),
}

/// ComplexAngle represents turns relative to lines
struct ComplexAngle {
    angle: i32,
    direction: AngleDirection,
}
/// Represents an Increasing or Decreasing angle
#[derive(PartialEq, Eq)]
enum AngleDirection {
    Increasing,
    Decreasing,
}

/// Transpose from one dimension to another, with the same aspect ratio
fn transpose(pos: Pos2, from: (f32, f32), to: (f32, f32)) -> Pos2 {
    // We assume the aspect ratio is the same
    let (from_w, from_h) = from;
    let (to_w, to_h) = to;
    Pos2 {
        x: (pos.x / from_w) * to_w,
        y: (pos.y / from_h) * to_h,
    }
}

impl From<bool> for AngleDirection {
    /// Converts from a bool representing if x is increasing to an AngleDirection
    fn from(value: bool) -> Self {
        if value {
            return Self::Increasing;
        }
        Self::Decreasing
    }
}

impl ComplexAngle {
    /// Construct a new complex angle with an angle and a boolean.
    fn new(angle: i32, increasing: bool) -> Self {
        // Limit the angle to a range of -90 to 90
        let mut res_angle = Self::normalize(angle);
        // Further limit range from -90 to 90
        // Since these angles are still bidirectional, it's fine to add/subtract 180
        if res_angle > 90 {
            res_angle = res_angle - 180;
        } else if res_angle < -90 {
            res_angle = res_angle + 180;
        }
        Self {
            angle: res_angle,
            direction: increasing.into(), // we already implement From<bool> for AngleDirection
        }
    }

    /// Normalizes an angle for more effecient turns. Sets the range of the angle from -180 to 180.
    fn normalize(angle: i32) -> i32 {
        // If angle is greater than 180, subtract 360
        if angle > 180 {
            return angle - 360;
        }
        if angle < -180 {
            return angle + 360;
        }
        // Otherwise, we're fine
        angle
    }

    /// Calculates a turn from self to supplied angle
    fn calculate_turn(&self, angle: &Self) -> i32 {
        // If direction's the same, subtract the angles
        if self.direction == angle.direction {
            return self.angle - angle.angle;
        }
        // If directions differ, subtract 180 from the result
        // And we normalize the angle for good measure
        return Self::normalize((self.angle - angle.angle) - 180);
    }
}

impl Default for PathyApp {
    fn default() -> Self {
        Self {
            // I actually set these to the 2023 Over-Under dimensions already,
            // but they can be changed nonetheless.
            height: 140.5,
            width: 140.5,
            scale: 720.0,
            mode: CursorMode::Default,
            path: Vec::new(),
            selected: 0,
            overlay: None,
            processed: Vec::new(),
            result: None,
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
    /// Preprocess the route to round all integers
    fn preprocess(path: &mut Vec<Pos2>, from: (f32, f32), to: (f32, f32)) -> Vec<Process> {
        /* To create the optimal route, we don't want to rely on somewhat imprecise and arbitrary
         * integers. Hence, all distances are rounded to the nearest inch. We also use the slope to
         * calculate the angle that the robot will need to be turned(again, rounded). Then, we'll
         * reconstruct a new path based off of these measurements, so that the changes can be
         * previewed before we generate the code.
         */

        //let mut path = path.clone();

        if path.len() < 2 {
            // This won't work, we need at least two points for a path
            return Vec::new();
        }
        // We'll need to transpose the path to the correct size
        *path = path.iter().map(|pos| transpose(*pos, from, to)).collect();

        // Original start
        let start: Pos2 = path[0];
        // Store the previous point
        let mut prev = path.remove(0);
        // Store the previous angle
        let mut prev_angle = ComplexAngle::new(90, true); // We start facing "up"
        let grouped_processes: Vec<(i32, i32, i32)> = path
            .iter()
            .map(|pos| {
                // Lets calculate the angle first.
                // First, we'll calculate the slope.
                // The slope is rise/run, and if we draw a triangle then we'll be able to see that it's // We can't serialize and image; and we don't want to
                // also the tangent of the angle a. Therefore, we'll just take the arc-tangent and round it
                // off.
                let cx = prev.x - pos.x;
                let cy = prev.y - pos.y;
                let slope = cy / cx; // Inverse since we're going from
                                     // TL origin to BL origin
                let mut angle: f32 = slope.atan().to_degrees();
                // Make complex angle - we check using >= since 90 degree angles are increasing
                // Distance is just pythagorean thm, and we just round it.
                let mut distance: f32 =
                //    f32::sqrt((pos.x - prev.x).powi(2) + (pos.x - prev.x).powi(2)).round() as i32;
                    cx / angle.to_radians().cos();
                if angle <= 1.0 && angle >= -1.0 {
                    // It's basically straight
                    angle = 0.0;
                }
                if cx == 0.0 {
                    // Things get buggy with 1/0, manual override
                    angle = 90.0;
                    distance = cy;
                }
                let complex_angle = ComplexAngle::new(-angle.round() as i32, pos.x >= prev.x);
                let turn: i32 = prev_angle.calculate_turn(&complex_angle);
                prev = *pos;
                prev_angle = complex_angle;
                (angle.round() as i32, distance.round() as i32, turn)
            })
            .collect();
        prev = start;
        // Transpose back
        *path = vec![transpose(start, to, from)];
        path.append(
            &mut grouped_processes
                .iter()
                .map(|(angle, distance, _)| {
                    // Calculate change in x
                    let mut cx: f32 = (*angle as f32).to_radians().cos() * (*distance as f32);
                    let mut cy: f32 = (*angle as f32).to_radians().sin() * (*distance as f32);
                    // Fix buggy 1/0
                    if *angle == 90 {
                        cx = 0.0;
                        cy = *distance as f32;
                    }
                    let result = Pos2 {
                        x: prev.x - cx,
                        y: prev.y - cy,
                    };
                    prev = result;
                    transpose(result, to, from)
                })
                .collect(),
        );
        grouped_processes
            .iter()
            .map(|(_, distance, turn)| [Process::Turn(*turn), Process::Drive(distance.abs())])
            .collect::<Vec<[Process; 2]>>()
            .as_slice()
            .concat()
    }
    fn generate(processes: &Vec<Process>) -> String {
        let mut result = String::from("// The following code was generated by Pathy:");
        processes.iter().for_each(|process| match *process {
            Process::Turn(angle) => {
                if angle != 0 {
                    // avoid unneccessary turns
                    result.push_str(
                        format!(
                            "\nchassis.set_turn_pid({}, TURN_SPEED)\nchassis.wait_drive();",
                            angle
                        )
                        .as_str(),
                    )
                }
            }
            Process::Drive(distance) => result.push_str(
                format!(
                    "\nchassis.set_drive_pid({}, DRIVE_SPEED)\nchassis.wait_drive();",
                    distance
                )
                .as_str(),
            ),
        });
        result
    }
}

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
                if ui.button("Preprocess").clicked() {
                    *processed = Self::preprocess(path, (*scale, aspecty), (*width, *height));
                }
                // Order here is important, as the ui button is only rendered if the first
                // condition is true. Otherwise, there's no point in evaluating the second
                // condition, thus not rendering the button.
                if processed.len() != 0 && ui.button("Generate").clicked() {
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
            ctx.input(|i| match i.raw.dropped_files.last() {
                Some(file) => match file.clone().bytes {
                    Some(bytes) => match RetainedImage::from_image_bytes("", &*bytes) {
                        Ok(image) => *overlay = Some(image),
                        Err(_) => (),
                    },
                    None => (),
                },
                None => (),
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
                match prev {
                    Some(prev_pos) => {
                        ui.painter()
                            .line_segment([prev_pos, screen_pos], yellow_line);
                    }
                    None => (),
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
                CursorMode::Edit | CursorMode::Delete | CursorMode::Trim => {
                    // Get pointer position
                    match hovered {
                        Some(hover_pos) => {
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
                        Some(idx) => drop(path.remove(idx)), // Deletes the elements
                        None => (),
                    },
                    CursorMode::Trim => match sl_idx {
                        Some(idx) => drop(path.drain(idx..)), // Deletes elements from idx
                        None => (),
                    },
                }
            }
            // Handle drags - Edit mode only
            if *mode == CursorMode::Edit && path.len() > 0 {
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
