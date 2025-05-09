use crate::DrawResult;
use plotters::coord::ranged3d::ProjectionMatrix;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use std::collections::BTreeMap;
use web_sys::HtmlCanvasElement;

static CTX: std::sync::Mutex<Option<Ctx>> = std::sync::Mutex::new(None);

pub fn gen_pyramid_surface() -> vdrm_alg::PixelSurface {
    let mut pixel_surface = vdrm_alg::PixelSurface::new();
    let r = vdrm_alg::W_PIXELS as i32 / 2;
    for x in 0..vdrm_alg::W_PIXELS as u32 {
        for y in 0..vdrm_alg::W_PIXELS as u32 {
            let x_i32 = x as i32 - r;
            let y_i32 = y as i32 - r;
            let h = x_i32.abs() + y_i32.abs();
            if h >= r {
                continue;
            }
            let z = (r - 1 - h) as u32;
            // let z = h as u32;
            let color = match (x_i32 >= 0, y_i32 >= 0) {
                (true, true) => 0b111,
                (false, true) => 0b001,
                (false, false) => 0b010,
                (true, false) => 0b101,
            };
            pixel_surface.push((x, y, (z, color)));
        }
    }
    pixel_surface
}
struct Mirror {
    points: [(f32, f32, f32); 4],
}
impl Mirror {
    fn new(len: f32, angle: u32) -> Self {
        let angle = vdrm_alg::angle_to_v(angle);
        let mat = glam::Mat2::from_angle(angle);
        let scal_w = std::f32::consts::SQRT_2;
        let scal_w = 1.;
        let points = [
            (
                len + vdrm_alg::SCREEN_OFFSET,
                len * scal_w,
                len * scal_w - len - vdrm_alg::SCREEN_OFFSET,
            ),
            (
                -len + vdrm_alg::SCREEN_OFFSET,
                len * scal_w,
                len - vdrm_alg::SCREEN_OFFSET,
            ),
            (
                -len + vdrm_alg::SCREEN_OFFSET,
                -len * scal_w,
                len - vdrm_alg::SCREEN_OFFSET,
            ),
            (
                len + vdrm_alg::SCREEN_OFFSET,
                -len * scal_w,
                -len - vdrm_alg::SCREEN_OFFSET,
            ),
        ];
        let points = points.map(|(x, y, z)| {
            let p = mat * glam::Vec2::new(x, y);
            (p.x, p.y, z)
        });
        Self { points }
    }
    fn polygon(&self) -> Polygon<(f32, f32, f32)> {
        Polygon::new(self.points, BLACK.mix(0.2))
    }
}

struct Screen {
    points: [(f32, f32, f32); 4],
}

impl Screen {
    fn new(idx: usize) -> Self {
        let xy_line = vdrm_alg::screens()[idx].xy_line;
        let (a, b) = xy_line.points();
        let points = [
            (a.x(), a.y(), -1. - vdrm_alg::SCREEN_OFFSET),
            (a.x(), a.y(), 1. - vdrm_alg::SCREEN_OFFSET),
            (b.x(), b.y(), 1. - vdrm_alg::SCREEN_OFFSET),
            (b.x(), b.y(), -1. - vdrm_alg::SCREEN_OFFSET),
        ];
        Self { points }
    }

    fn polygon(&self) -> Polygon<(f32, f32, f32)> {
        Polygon::new(self.points, BLACK.mix(0.8))
    }
}

struct AngleCtx {
    mirror: Mirror,
    led_pixels: Vec<(f32, f32, f32)>,
    emu_pixels: Vec<(f32, f32, f32)>,
}

#[derive(Clone, PartialEq, Eq)]
struct CtxParam {
    angle_range: std::ops::Range<usize>,
    enb_screens: Vec<usize>,
}
struct Ctx {
    angle_ctx_map: BTreeMap<u32, AngleCtx>,
    all_real_pixels: Vec<(f32, f32, f32)>,
    all_emu_pixels: Vec<(f32, f32, f32)>,
    all_led_pixels: Vec<(f32, f32, f32)>,
    screens: [Screen; 3],
    param: CtxParam,
}

impl Ctx {
    fn new(param: CtxParam) -> Self {
        let codec = vdrm_alg::Codec::new(param.angle_range.clone());
        let pixel_surface = gen_pyramid_surface();
        let all_real_pixels = vdrm_alg::pixel_surface_to_float(&pixel_surface)
            .into_iter()
            .map(|(x, y, z)| {
                (
                    x,
                    y + vdrm_alg::SCREEN_OFFSET,
                    -z - 1. - vdrm_alg::SCREEN_OFFSET,
                )
            })
            .collect();
        let optimze_speed_for_mbi5264 = false;
        let angle_map = codec.encode(&pixel_surface, 0, optimze_speed_for_mbi5264);
        let (mut all_emu_pixels, mut all_led_pixels) = (vec![], vec![]);
        let angle_ctx_map = (0..vdrm_alg::TOTAL_ANGLES as u32)
            .map(|angle| {
                let mirror = Mirror::new(1. / 2_f32.sqrt(), angle);
                let Some(lines_arr) = angle_map.get(&angle) else {
                    return (
                        angle,
                        AngleCtx {
                            mirror,
                            led_pixels: vec![],
                            emu_pixels: vec![],
                        },
                    );
                };
                let mut led_pixels = vec![];
                let mut emu_pixels = vec![];
                for (idx, lines) in lines_arr.iter().enumerate() {
                    if !param.enb_screens.contains(&idx) {
                        continue;
                    }
                    let (emu_pixels_dec, led_pixels_dec) = codec.decode(angle, lines);
                    all_emu_pixels.extend(emu_pixels_dec.clone());
                    all_led_pixels.extend(led_pixels_dec.clone());
                    led_pixels.extend(led_pixels_dec);
                    emu_pixels.extend(emu_pixels_dec);
                }
                let angle_ctx = AngleCtx {
                    mirror,
                    led_pixels,
                    emu_pixels,
                };
                (angle, angle_ctx)
            })
            .collect();

        Self {
            angle_ctx_map,
            all_real_pixels,
            all_emu_pixels,
            all_led_pixels,
            screens: [0, 1, 2].map(|idx| Screen::new(idx)),
            param,
        }
    }
}

pub fn draw(
    canvas: HtmlCanvasElement,
    angle: Option<u32>,
    pitch: f64,
    yaw: f64,
    angle_range: std::ops::Range<usize>,
    enb_screens: Vec<usize>,
) -> DrawResult<()> {
    static INIT_LOG: std::sync::Once = std::sync::Once::new();
    INIT_LOG.call_once(|| {
        wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
        console_error_panic_hook::set_once();
    });
    log::info!("draw");
    let mut guard = CTX.lock().unwrap();
    let param = CtxParam {
        angle_range,
        enb_screens,
    };
    let ctx = guard.get_or_insert_with(|| Ctx::new(param.clone()));
    if ctx.param != param {
        *ctx = Ctx::new(param);
    }
    let area = CanvasBackend::with_canvas_object(canvas)
        .unwrap()
        .into_drawing_area();
    area.fill(&WHITE)?;

    let axis_len = 1.5f32;
    let x_axis = (-axis_len..axis_len).step(0.1);
    let y_axis = (0f32..axis_len * 2.).step(0.1);

    let mut chart = ChartBuilder::on(&area).build_cartesian_3d(
        x_axis.clone(),
        y_axis.clone(),
        -axis_len * 2. ..0.,
    )?;
    chart.with_projection(|_pb| {
        let (x, y) = area.get_pixel_range();
        let v = (x.end - x.start).min(y.end - y.start) * 4 / 5 / 2;
        let before = (v, v, v);
        let after = ((x.start + x.end) / 2, (y.start + y.end) / 2);

        let mut mat = if before == (0, 0, 0) {
            ProjectionMatrix::default()
        } else {
            let (x, y, z) = before;
            ProjectionMatrix::shift(-x as f64, -y as f64, -z as f64) * ProjectionMatrix::default()
        };
        if yaw.abs() > 1e-20 {
            mat = mat * ProjectionMatrix::rotate(0.0, 0.0, yaw);
        }
        if pitch.abs() > 1e-20 {
            mat = mat * ProjectionMatrix::rotate(pitch, 0.0, 0.0);
        }
        mat = mat * ProjectionMatrix::scale(0.7);
        if after != (0, 0) {
            let (x, y) = after;
            mat = mat * ProjectionMatrix::shift(x as f64, y as f64, 0.0);
        }
        mat
    });

    chart.configure_axes().draw()?;

    chart
        .draw_series(
            [
                ("x", (axis_len, 0., -axis_len * 2.), &RED),
                ("y", (-axis_len, axis_len * 2., -axis_len * 2.), &GREEN),
                ("z", (-axis_len, 0., 0.), &BLUE),
                ("o'", (-axis_len, 0., -axis_len * 2.), &CYAN),
            ]
            .map(|(label, position, color)| {
                Text::new(
                    label,
                    position,
                    ("sans-serif", 20, color).into_text_style(&area),
                )
            }),
        )
        .unwrap();
    let screen_polygons = ctx.screens.iter().map(|v| v.polygon());
    chart
        .draw_series(screen_polygons)?
        .label("SCREEN")
        .legend(|(x, y)| {
            Rectangle::new([(x + 5, y - 5), (x + 15, y + 5)], BLACK.mix(0.9).filled())
        });
    let real_surface_points: PointSeries<_, _, Circle<_, _>, _> =
        PointSeries::new(ctx.all_real_pixels.clone(), 1_f64, &BLUE.mix(0.2));
    chart
        .draw_series(real_surface_points)?
        .label("REAL")
        .legend(|(x, y)| Rectangle::new([(x + 5, y - 5), (x + 15, y + 5)], BLUE.mix(0.5).filled()));

    let (emu, led) = match angle {
        None => (ctx.all_emu_pixels.clone(), ctx.all_led_pixels.clone()),
        Some(angle) => {
            let angle_ctx = ctx.angle_ctx_map.get(&angle).unwrap();
            chart
                .draw_series([angle_ctx.mirror.polygon()])?
                .label("MIRROR")
                .legend(|(x, y)| {
                    Rectangle::new([(x + 5, y - 5), (x + 15, y + 5)], BLACK.mix(0.5).filled())
                });

            (angle_ctx.emu_pixels.clone(), angle_ctx.led_pixels.clone())
        }
    };

    let emu_surface_points: PointSeries<_, _, Circle<_, _>, _> =
        PointSeries::new(emu, 1_f32, &RED.mix(0.3));
    chart
        .draw_series(emu_surface_points)?
        .label("EMULATOR")
        .legend(|(x, y)| Rectangle::new([(x + 5, y - 5), (x + 15, y + 5)], RED.mix(0.5).filled()));

    let led_surface_points: PointSeries<_, _, Circle<_, _>, _> =
        PointSeries::new(led, 1_f64, &RED.mix(0.8));
    chart.draw_series(led_surface_points)?;

    chart.configure_series_labels().border_style(BLACK).draw()?;
    Ok(())
}
