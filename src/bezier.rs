use std::{cell::RefCell, rc::Rc};

use egui::{lerp, pos2, Color32, Context, Pos2, Stroke, Ui};
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
/// A Bezier point.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct BezPoint {
    pub pos: Rc<RefCell<Point>>,
    pub cp1: Rc<RefCell<Point>>,
    pub cp2: Rc<RefCell<Point>>,

    // ID
    id: Uuid,
    // Previous position to keep track of offsets
    prev: Point,
}

/// A single selectable point.
#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub selected: bool,
    pub locked: bool,
}

impl Point {
    /// Creates a new point.
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            selected: false,
            locked: false,
        }
    }
    /// Offsets the point by the x and y.
    fn offset(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }
}

impl BezPoint {
    /// Creates a new bezier point.
    pub fn new(x: f32, y: f32, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32) -> Self {
        Self {
            pos: Rc::new(RefCell::new(Point::new(x, y))),
            cp1: Rc::new(RefCell::new(Point::new(cp1x, cp1y))),
            cp2: Rc::new(RefCell::new(Point::new(cp2x, cp2y))),
            id: Uuid::new_v4(),
            prev: Point::new(x, y),
        }
    }
    /// Draws the bezier point and handles, handling animations and hover states.
    /// If hovered, returns the hovered point.
    pub fn draw(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        ratio: f32,
        origin: Pos2,
        hover_pos: Option<Pos2>,
    ) -> Option<Rc<RefCell<Point>>> {
        let r = 5.0; // point radius
        let r_hov = 8.0; // hover radius

        // Generate an id to keep track of point animations
        let id = ui.make_persistent_id(self.id);
        let p_id = id.with(0);
        let cp1_id = id.with(1);
        let cp2_id = id.with(2);

        // Keep control points in line
        if self.pos.borrow().locked {
            let dx = self.pos.borrow().x - self.prev.x;
            let dy = self.pos.borrow().y - self.prev.y;
            self.cp1.borrow_mut().offset(dx, dy);
            self.cp2.borrow_mut().offset(dx, dy);
            self.prev = self.pos.borrow().clone();
        } else if self.cp1.borrow().locked {
            *self.cp2.borrow_mut() = Point::new(
                2.0 * self.pos.borrow().x - self.cp1.borrow().x,
                2.0 * self.pos.borrow().y - self.cp1.borrow().y,
            );
        } else if self.cp2.borrow().locked {
            *self.cp1.borrow_mut() = Point::new(
                2.0 * self.pos.borrow().x - self.cp2.borrow().x,
                2.0 * self.pos.borrow().y - self.cp2.borrow().y,
            );
        }

        // Main point
        let x = self.pos.borrow().x * ratio + origin.x;
        let y = self.pos.borrow().y * ratio + origin.y;
        // Control points
        let cp1x = self.cp1.borrow().x * ratio + origin.x;
        let cp1y = self.cp1.borrow().y * ratio + origin.y;
        let cp2x = self.cp2.borrow().x * ratio + origin.x;
        let cp2y = self.cp2.borrow().y * ratio + origin.y;

        #[derive(Clone)]
        enum Selected {
            P,
            CP1,
            CP2,
        }

        // Update hover states
        self.pos.borrow_mut().selected = false;
        self.cp1.borrow_mut().selected = false;
        self.cp2.borrow_mut().selected = false;
        let mut selected: Option<Selected> = None;
        if let Some(hover_pos) = hover_pos {
            let mut distances = [
                (
                    Selected::CP1,
                    &mut self.cp1.borrow_mut().selected,
                    hover_pos.distance_sq(pos2(cp1x, cp1y)),
                ),
                (
                    Selected::CP2,
                    &mut self.cp2.borrow_mut().selected,
                    hover_pos.distance_sq(pos2(cp2x, cp2y)),
                ),
            ];
            let mut point_dis = (
                Selected::P,
                &mut self.pos.borrow_mut().selected,
                hover_pos.distance_sq(pos2(x, y)),
            );
            let (point, hovered, min_distance) = distances
                .iter_mut()
                .fold(&mut point_dis, |min, x| if x.2 < min.2 { x } else { min });
            if *min_distance < r * r {
                **hovered = true;
                selected = Some(point.clone());
            } else {
                **hovered = false;
            }
        }

        // Update point radii based on hover state
        let p_r = lerp(
            r..=r_hov,
            ctx.animate_bool(p_id, self.pos.borrow().selected || self.pos.borrow().locked),
        );
        let cp1_r = lerp(
            r..=r_hov,
            ctx.animate_bool(
                cp1_id,
                self.cp1.borrow().selected || self.cp1.borrow().locked,
            ),
        );
        let cp2_r = lerp(
            r..=r_hov,
            ctx.animate_bool(
                cp2_id,
                self.cp2.borrow().selected || self.cp2.borrow().locked,
            ),
        );

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
        match selected {
            Some(Selected::P) => Some(self.pos.clone()),
            Some(Selected::CP1) => Some(self.cp1.clone()),
            Some(Selected::CP2) => Some(self.cp2.clone()),
            None => None,
        }
    }
}
