use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::app::CursorMode;
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
#[derive(Clone, Debug)]
pub struct BezPoint {
    pub pos: Rc<RefCell<Point>>,
    pub cp1: Rc<RefCell<Point>>,
    pub cp2: Rc<RefCell<Point>>,

    // ID
    pub id: Uuid,
    pub animated: bool,
    // Previous position to keep track of offsets
    prev: Point,
    // If the path "breaks" here (cusp, not tangent)
    pub broken: bool,
}

/// A single selectable point.
#[derive(Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub selected: bool,
    pub locked: bool,
    pub editing: bool,
    pub parent: Weak<RefCell<BezPoint>>,
}

/// A saved Bezier point.
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct SavePoint {
    pub pos: Pos2,
    pub cp1: Pos2,
    pub cp2: Pos2,
    pub id: Uuid,
    pub broken: bool,
}

impl Point {
    /// Creates a new point.
    fn new(x: f32, y: f32, parent: Weak<RefCell<BezPoint>>) -> Self {
        Self {
            x,
            y,
            selected: false,
            locked: false,
            editing: false,
            parent,
        }
    }
    /// Offsets the point by the x and y.
    pub fn offset(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }
    /// Gets the screen position
    pub fn screen(&self, ratio: f32, origin: Pos2) -> Pos2 {
        pos2(self.x * ratio + origin.x, self.y * ratio + origin.y)
    }
}

impl From<Point> for Pos2 {
    fn from(point: Point) -> Self {
        pos2(point.x, point.y)
    }
}

impl From<BezPoint> for SavePoint {
    /// Saves a BezPoint as a SavePoint.
    fn from(point: BezPoint) -> Self {
        Self {
            pos: point.pos.borrow().clone().into(),
            cp1: point.cp1.borrow().clone().into(),
            cp2: point.cp2.borrow().clone().into(),
            id: point.id,
            broken: point.broken,
        }
    }
}

impl From<SavePoint> for BezPoint {
    /// Returns an unloaded Bezier point from a saved point.
    fn from(point: SavePoint) -> Self {
        Self {
            pos: Rc::new(RefCell::new(Point::new(
                point.pos.x,
                point.pos.y,
                Weak::new(),
            ))),
            cp1: Rc::new(RefCell::new(Point::new(
                point.cp1.x,
                point.cp1.y,
                Weak::new(),
            ))),
            cp2: Rc::new(RefCell::new(Point::new(
                point.cp2.x,
                point.cp2.y,
                Weak::new(),
            ))),
            id: point.id,
            animated: true,
            prev: Point::new(point.pos.x, point.pos.y, Weak::new()),
            broken: point.broken,
        }
    }
}

impl BezPoint {
    /// Creates a new bezier point.
    ///
    /// # Arguments
    /// * `x` - The x position of the center point.
    /// * `y` - The y position of the center point.
    /// * `cp1x` - The x position of the first control point.
    /// * `cp1y` - The y position of the first control point.
    /// * `cp2x` - The x position of the second control point.
    /// * `cp2y` - The y position of the second control point.
    pub fn new(x: f32, y: f32, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32) -> Rc<RefCell<Self>> {
        Self::load(Self {
            pos: Rc::new(RefCell::new(Point::new(x, y, Weak::new()))),
            cp1: Rc::new(RefCell::new(Point::new(cp1x, cp1y, Weak::new()))),
            cp2: Rc::new(RefCell::new(Point::new(cp2x, cp2y, Weak::new()))),
            id: Uuid::new_v4(),
            animated: false,
            prev: Point::new(x, y, Weak::new()),
            broken: false,
        })
    }
    /// Creates a new Bezier point from a saved one, instantiating its references.
    ///
    /// # Arguments
    /// * `point` - The point to load.
    pub fn load(point: Self) -> Rc<RefCell<Self>> {
        let this = Rc::new(RefCell::new(point.clone()));
        this.borrow_mut().pos.borrow_mut().parent = Rc::downgrade(&this);
        this.borrow_mut().cp1.borrow_mut().parent = Rc::downgrade(&this);
        this.borrow_mut().cp2.borrow_mut().parent = Rc::downgrade(&this);
        this.borrow_mut().prev.parent = Rc::downgrade(&this);
        this
    }
    /// Draws the bezier point and handles, handling animations and hover states.
    /// If hovered, returns the hovered point.
    ///
    /// # Arguments
    /// * `ui` - The egui ui.
    /// * `ctx` - The egui context.
    /// * `ratio` - The ratio of the screen size to the field size.
    /// * `origin` - The origin of the field(top-left corner).
    /// * `mode` - The current cursor mode. CursorMode::Trim should only be supplied to points which will be deleted.
    /// * `hover_pos` - The position of the cursor.
    ///
    /// # Returns
    /// `Some(Rc<RefCell<Point>>)` containing the hovered point, or None if no point is hovered.
    pub fn draw(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        ratio: f32,
        origin: Pos2,
        mode: &CursorMode,
        hover_pos: Option<Pos2>,
    ) -> Option<Rc<RefCell<Point>>> {
        let r = 5.0; // point radius
        let r_hov = 8.0; // hover radius

        // Generate an id to keep track of point animations
        let id = ui.make_persistent_id(self.id);
        let p_id = id.with(0);
        let cp1_id = id.with(1);
        let cp2_id = id.with(2);

        // Keep control points in line if unbroken
        if self.pos.borrow().locked || self.pos.borrow().editing {
            let dx = self.pos.borrow().x - self.prev.x;
            let dy = self.pos.borrow().y - self.prev.y;
            self.cp1.borrow_mut().offset(dx, dy);
            self.cp2.borrow_mut().offset(dx, dy);
            self.prev = self.pos.borrow().clone();
        } else if !self.broken
            && (self.cp1.borrow().locked || self.cp1.borrow().editing || self.pos.borrow().editing)
        {
            self.cp2.borrow_mut().x = 2.0 * self.pos.borrow().x - self.cp1.borrow().x;
            self.cp2.borrow_mut().y = 2.0 * self.pos.borrow().y - self.cp1.borrow().y;
        } else if !self.broken && (self.cp2.borrow().locked || self.cp2.borrow().editing) {
            self.cp1.borrow_mut().x = 2.0 * self.pos.borrow().x - self.cp2.borrow().x;
            self.cp1.borrow_mut().y = 2.0 * self.pos.borrow().y - self.cp2.borrow().y;
        }

        // Ensure points are in line
        if !self.broken {
            self.cp2.borrow_mut().x = 2.0 * self.pos.borrow().x - self.cp1.borrow().x;
            self.cp2.borrow_mut().y = 2.0 * self.pos.borrow().y - self.cp1.borrow().y;
        }

        // Main point
        let Pos2 { x, y } = self.pos.borrow().screen(ratio, origin);
        // Control points
        let Pos2 { x: cp1x, y: cp1y } = self.cp1.borrow().screen(ratio, origin);
        let Pos2 { x: cp2x, y: cp2y } = self.cp2.borrow().screen(ratio, origin);

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
        let dont_select = matches!(
            *mode,
            CursorMode::Delete | CursorMode::Trim | CursorMode::Insert
        );
        let p_r = lerp(
            r..=r_hov,
            ctx.animate_bool(
                p_id,
                !dont_select && (self.pos.borrow().selected || self.pos.borrow().locked),
            ),
        );
        let cp1_r = lerp(
            r..=r_hov,
            ctx.animate_bool(
                cp1_id,
                !dont_select && (self.cp1.borrow().selected || self.cp1.borrow().locked),
            ),
        );
        let cp2_r = lerp(
            r..=r_hov,
            ctx.animate_bool(
                cp2_id,
                !dont_select && (self.cp2.borrow().selected || self.cp2.borrow().locked),
            ),
        );
        let color = Color32::YELLOW.lerp_to_gamma(
            Color32::RED,
            ctx.animate_bool(
                id,
                (*mode == CursorMode::Trim)
                    || ((*mode == CursorMode::Delete)
                        && (self.pos.borrow().selected
                            || self.cp1.borrow().selected
                            || self.cp2.borrow().selected)),
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
            Stroke::new(2.0, color),
        );
        ui.painter().line_segment(
            [pos2(x, y), pos2(cp2x - xoffset2, cp2y - yoffset2)],
            Stroke::new(2.0, color),
        );

        // Draw points
        ui.painter().circle_filled(pos2(x, y), p_r, color);
        ui.painter()
            .circle_stroke(pos2(cp1x, cp1y), cp1_r, Stroke::new(2.0, color));
        ui.painter()
            .circle_stroke(pos2(cp2x, cp2y), cp2_r, Stroke::new(2.0, color));
        match selected {
            Some(Selected::P) => Some(self.pos.clone()),
            Some(Selected::CP1) => Some(self.cp1.clone()),
            Some(Selected::CP2) => Some(self.cp2.clone()),
            None => None,
        }
    }
}

/// Find the in-between point of a Bezier curve section at t, where t is from [0, 1].
pub fn interpolate(a: &BezPoint, b: &BezPoint, t: f32) -> Point {
    let x = (1.0 - t).powi(3) * a.pos.borrow().x
        + 3.0 * (1.0 - t).powi(2) * t * a.cp2.borrow().x
        + 3.0 * (1.0 - t) * t.powi(2) * b.cp1.borrow().x
        + t.powi(3) * b.pos.borrow().x;
    let y = (1.0 - t).powi(3) * a.pos.borrow().y
        + 3.0 * (1.0 - t).powi(2) * t * a.cp2.borrow().y
        + 3.0 * (1.0 - t) * t.powi(2) * b.cp1.borrow().y
        + t.powi(3) * b.pos.borrow().y;
    Point::new(x, y, Weak::new())
}

/*
/// Find the in-between slope of a Bezier curve section at t, where t is from [0, 1].
/// # Returns
/// Some(f32) if slope is defined, None if slope is undefined (vertical).
pub fn interpolate_slope(a: &BezPoint, b: &BezPoint, t: f32) -> Option<f32> {
    let dx = 3.0 * (1.0 - t).powi(2) * (a.cp2.borrow().x - a.pos.borrow().x)
        + 6.0 * (1.0 - t) * t * (b.cp1.borrow().x - a.cp2.borrow().x)
        + 3.0 * t.powi(2) * (b.pos.borrow().x - b.cp1.borrow().x);
    if dx.abs() < 1e-6 {
        // check against epsilon
        return None;
    }
    let dy = 3.0 * (1.0 - t).powi(2) * (a.cp2.borrow().y - a.pos.borrow().y)
        + 6.0 * (1.0 - t) * t * (b.cp1.borrow().y - a.cp2.borrow().y)
        + 3.0 * t.powi(2) * (b.pos.borrow().y - b.cp1.borrow().y);
    return Some(dy / dx);
}
*/
