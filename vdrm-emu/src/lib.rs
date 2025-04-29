use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

mod plot3d;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Type alias for the result of a drawing function.
pub type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Type used on the JS side to convert screen coordinates to chart
/// coordinates.
#[wasm_bindgen]
pub struct Chart {}

#[wasm_bindgen]
impl Chart {
    pub fn plot3d(
        canvas: HtmlCanvasElement,
        angle: Option<u32>,
        pitch: f64,
        yaw: f64,
        min_angle: usize,
        max_angle: usize,
        enb_screens: Vec<usize>,
    ) -> Result<(), JsValue> {
        let angle_range = min_angle..max_angle;
        plot3d::draw(canvas, angle, pitch, yaw, angle_range, enb_screens).map_err(|err| err.to_string())?;
        Ok(())
    }
}
