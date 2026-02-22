use std::f32::consts::PI;
const MIRROR_OFFSET: f32 = 1.;
const SCREEN_OFFSET: f32 = 0.25;
const SCREEN_X_OFFSET: f32 = -1.0 + SCREEN_OFFSET;
const NUM_SCREENS: usize = 3;
use crate::DrawResult;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use web_sys::HtmlCanvasElement;

fn mirror_mat3(angle_f: f32) -> glam::Mat3A {
    let sin = angle_f.sin();
    let cos = angle_f.cos();
    let sin2 = sin * sin;
    let cos2 = cos * cos;
    let sin_cos = sin * cos;
    let mat_mir = glam::Mat3A::from_cols(
        glam::Vec3A::new(sin2 - cos2, -2.0 * sin_cos, 0.),
        glam::Vec3A::new(-2.0 * sin_cos, cos2 - sin2, 0.),
        glam::Vec3A::new(2.0 * MIRROR_OFFSET * cos, 2.0 * MIRROR_OFFSET * sin, 1.),
    );
    mat_mir
}

fn mirror_points_f(angle_f: f32, points: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mat = mirror_mat3(angle_f);
    points
        .iter()
        .map(|v| {
            let v = glam::Vec3A::new(v.0, v.1, 1.0);
            let v = mat * v;
            (v.x, v.y)
        })
        .collect()
}

pub fn screens() -> [[(f32, f32); 2]; NUM_SCREENS] {
    let depth = 1f32;
    let a: (f32, f32) = (SCREEN_X_OFFSET, 0.);

    let b: (f32, f32) = (SCREEN_X_OFFSET + depth, 0.);

    let screen = [a, b];
    let angle = 180f32;
    let angle_off = 360.0f32 / 8.0 / 2.0 / 2.0;
    let angle_l = angle - angle_off;
    let angle_r = angle + angle_off;
    let v_screen = mirror_points_f(angle.to_radians(), &screen);
    let screen_l = mirror_points_f(angle_l.to_radians(), &v_screen);
    let screen_r = mirror_points_f(angle_r.to_radians(), &v_screen);

    let screen_l = screen_l.try_into().unwrap();
    let screen_r = screen_r.try_into().unwrap();
    [screen_l, screen, screen_r]
}

/// Draw power function f(x) = x^power.
pub fn draw(
    canvas: HtmlCanvasElement,
    angle_offset: u32,
    enb_screens: Vec<usize>,
) -> DrawResult<impl Fn((i32, i32)) -> Option<(f32, f32)>> {
    let backend = CanvasBackend::with_canvas_object(canvas).unwrap();
    let root = backend.into_drawing_area();
    let font: FontDesc = ("sans-serif", 20.0).into();
    let cord_len = 3f32;

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20u32)
        .caption(format!("2d simulate"), font)
        .x_label_area_size(30u32)
        .y_label_area_size(30u32)
        .build_cartesian_2d(-cord_len..cord_len, -cord_len..cord_len)?;

    chart.configure_mesh().x_labels(3).y_labels(3).draw()?;

    let angle_offset = (angle_offset as f32 / 2. - 22.5).to_radians();
    let angle_init = PI / 8.0;
    let r = MIRROR_OFFSET / (PI / 8.0).cos();
    let mut points: Vec<_> = (0..8)
        .map(|i| {
            let angle = PI * i as f32 / 4. + angle_init + angle_offset;
            let x = r * angle.cos();
            let y = r * angle.sin();
            (x, y)
        })
        .collect();
    points.push(points[0]);
    chart.draw_series(LineSeries::new(points, &BLACK))?;
    let angle = PI + angle_offset;
    for (idx, screen) in screens().into_iter().enumerate() {
        if !enb_screens.contains(&idx) {
            continue;
        }
        for offset in [-PI / 4., 0f32, PI / 4.] {
            let angle = angle + offset;
            let v_screen = mirror_points_f(angle, &screen);
            chart.draw_series(LineSeries::new(v_screen, &BLACK.mix(0.5)))?;
        }

        chart.draw_series(LineSeries::new(screen, &BLACK))?;
    }

    root.present()?;
    return Ok(chart.into_coord_trans());
}
