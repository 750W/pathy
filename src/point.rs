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
/// A single selectable point.
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub selected: bool,
    pub locked: bool, // locks selection in case of dragging
    pub animated: bool,
    pub id: Uuid,
}

impl Point {
    const RADIUS: f32 = 5.0;
    const HOVER_RADIUS: f32 = 8.0;
    /// Creates a new point.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            selected: false,
            locked: false,
            animated: false,
            id: Uuid::new_v4(),
        }
    }
    /// Offsets the point by the x and y.
    pub fn offset(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }
    /// Gets the screen position
    pub fn screen(&self, ratio: f32, origin: Pos2) -> Pos2 {
        self.screen_clone(ratio, origin).into()
    }

    /// Gets screen position in a new Point
    pub fn screen_clone(&self, ratio: f32, origin: Pos2) -> Point {
        Self {
            x: self.x * ratio + origin.x,
            y: self.y * ratio + origin.y,
            selected: self.selected,
            locked: self.locked,
            animated: self.animated,
            id: self.id,
        }
    }

    /// Draws the point, handling animations and hover states.
    /// If hovered, returns true, otherwise returns false.
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
    /// If hovered, returns true, otherwise returns false.
    pub fn draw(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        ratio: f32,
        origin: Pos2,
        mode: &CursorMode,
        hover_pos: Option<Pos2>,
    ) -> bool {
        // Generate an id to keep track of point animations
        let id = ui.make_persistent_id(self.id);

        let Pos2 { x, y } = self.screen(ratio, origin);

        // Update hover state
        if let Some(hover_pos) = hover_pos {
            let point_dis = hover_pos.distance_sq(pos2(x, y));
            if point_dis < Self::RADIUS * Self::RADIUS {
                self.selected = true;
            } else {
                self.selected = false;
            }
        }

        let active = self.selected || self.locked;

        // Update point radii based on hover state
        let dont_select = matches!(
            *mode,
            CursorMode::Delete | CursorMode::Trim | CursorMode::Insert
        );
        let radius = lerp(
            Self::RADIUS..=Self::HOVER_RADIUS,
            ctx.animate_bool(id, !dont_select && active),
        );
        let color = Color32::YELLOW.lerp_to_gamma(
            Color32::RED,
            ctx.animate_bool(
                id.with(0),
                (*mode == CursorMode::Trim) || ((*mode == CursorMode::Delete) && self.selected),
            ),
        );

        // Draw point
        ui.painter()
            .circle_stroke(pos2(x, y), radius, Stroke::new(2.0, color));
        console_log!("{}", self.selected);
        self.selected
    }

    pub fn get_radius(&self) -> f32 {
        console_log!("{}", self.selected || self.locked);
        if self.selected || self.locked {
            Self::HOVER_RADIUS
        } else {
            Self::RADIUS
        }
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

pub fn interpolate(p1: &Point, p2: &Point, t: f32) -> Point {
    let pos1 = Pos2::from(p1.clone());
    let pos2 = Pos2::from(p2.clone());
    let direction = (pos2 - pos1).normalized();
    let p1_inner = pos1 + direction * p1.get_radius();
    let p2_inner = pos2 - direction * p2.get_radius();
    let x = lerp(p1_inner.x..=p2_inner.x, t);
    let y = lerp(p1_inner.y..=p2_inner.y, t);
    Point::new(x, y)
}

impl From<Point> for Pos2 {
    fn from(point: Point) -> Self {
        pos2(point.x, point.y)
    }
}
