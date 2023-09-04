use egui::Pos2;
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
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PathyApp {
    // this how you opt-out of serialization of a member
    //#[serde(skip)]
    pub height: f32,             // Height of field
    pub width: f32,              // Width of field
    pub scale: f32,              // Scale to display
    pub mode: CursorMode,        // Cursor mode
    pub path: Vec<Pos2>,         // Current path
    pub selected: usize,         // Current selected node (edit mode)
    pub processed: Vec<Process>, // Processed fields
    #[serde(skip)] // We can't serialize and image; and we don't want to
    pub overlay: Option<RetainedImage>, // Uploaded overlay
    pub result: Option<String>,  // Final string
}

#[derive(serde::Deserialize, serde::Serialize, Eq, PartialEq)]
pub enum CursorMode {
    // Represent possible cursor modes
    Default, // No action
    Create,  // Create new nodes and paths
    Edit,    // Edit the positioning of points
    Delete,  // Delete points (whilst still keeping only one path).
    Trim,    // Trim points (whilst keeping one path)
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum Process {
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
            res_angle -= 180;
        } else if res_angle < -90 {
            res_angle += 180;
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
        Self::normalize((self.angle - angle.angle) - 180)
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
    pub fn preprocess(path: &mut Vec<Pos2>, from: (f32, f32), to: (f32, f32)) -> Vec<Process> {
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
        // Store the previous angle - we start facing "up" might make it an option later
        let mut prev_angle = ComplexAngle::new(90, true);
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
                let mut complex_angle = ComplexAngle::new(-angle.round() as i32, pos.x >= prev.x);
                if cx == 0.0 {
                    // Things get buggy with 1/0, manual override
                    if cy.is_sign_positive() {
                        complex_angle.angle = 90;
                    } else {
                        complex_angle.angle = -90;
                    }
                    angle = 90.0;
                    distance = cy;
                }
                let mut turn: i32 = prev_angle.calculate_turn(&complex_angle);
                if (-1..=1).contains(&turn) {
                    // It's basically straight
                    turn = 0;
                }
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
    pub fn generate(processes: &[Process]) -> String {
        let mut result = String::from("// The following code was generated by Pathy:");
        processes.iter().for_each(|process| match *process {
            Process::Turn(angle) => {
                if angle != 0 {
                    // avoid unneccessary turns
                    result.push_str(
                        format!(
                            "\nchassis.set_turn_pid({}, TURN_SPEED);\nchassis.wait_drive();",
                            angle
                        )
                        .as_str(),
                    )
                }
            }
            Process::Drive(distance) => result.push_str(
                format!(
                    "\nchassis.set_drive_pid({}, DRIVE_SPEED);\nchassis.wait_drive();",
                    distance
                )
                .as_str(),
            ),
        });
        result
    }
}
