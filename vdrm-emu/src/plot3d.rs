use crate::DrawResult;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use web_sys::HtmlCanvasElement;


lazy_static::lazy_static! {
    static ref SURFACES: Surfaces = {
        Surfaces::new()
    };
}

pub fn gen_pyramid_surface() -> vdrm_alg::PixelSurface {
    let mut pixel_surface = vdrm_alg::PixelSurface::new();
    for x in 0..64_u32 {
        for y in 0..64_u32 {
            let x_i32 = x as i32 - 32;
            let y_i32 = y as i32 - 32;
            let h = 32 - (x_i32.abs() + y_i32.abs());
            if h < 0 {
                continue;
            }
            let z = h.abs() as u32;
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

struct Surfaces {
    real: vdrm_alg::FloatSurface,
    emu: vdrm_alg::FloatSurface,
}

impl Surfaces {
    fn new() -> Self {
        let codec = vdrm_alg::Codec::new();
        let pixel_surface = gen_pyramid_surface();
        let real = vdrm_alg::pixel_surface_to_float(&pixel_surface).into_iter().map(|(x, y, z)|(x, z - 2., y)).collect();
        let angle_map = codec.encode(&pixel_surface, 0);
        let emu = codec.decode_all(angle_map).into_iter().map(|(x, y, z)|(x, z, y)).collect();
        Self {
            real,
            emu,
        }
    }
}

pub fn draw(canvas: HtmlCanvasElement, pitch: f64, yaw: f64) -> DrawResult<()> {
    let area = CanvasBackend::with_canvas_object(canvas)
        .unwrap()
        .into_drawing_area();
    area.fill(&WHITE)?;

    let axis_len = 1.5_f32;
    let x_axis = (-axis_len..axis_len).step(0.1);
    let z_axis = (-axis_len..axis_len).step(0.1);

    let mut chart =
        ChartBuilder::on(&area).build_cartesian_3d(x_axis.clone(), -axis_len..axis_len, z_axis.clone())?;

    chart.with_projection(|mut pb| {
        pb.yaw = yaw;
        pb.pitch = pitch;
        pb.scale = 0.7;
        pb.into_matrix()
    });

    chart.configure_axes().draw()?;

    let axis_title_style = ("sans-serif", 20, &BLACK).into_text_style(&area);
    chart
        .draw_series(
            [
                ("x", (axis_len, -axis_len, -axis_len)),
                ("y", (-axis_len, axis_len, -axis_len)),
                ("z", (-axis_len, -axis_len, axis_len)),
            ]
            .map(|(label, position)| Text::new(label, position, &axis_title_style)),
        )
        .unwrap();

    // let mut line_points = vec![];
    // for line in line_points.clone() {
    //     chart.draw_series(LineSeries::new(line, &BLACK))?;
    // }
    //
    // let x_axis = (-1.0..1.0).step(0.1);
    // let z_axis = (-1.0..1.0).step(0.1);

    // let low_surface = SurfaceSeries::xoz(x_axis.values(), z_axis.values(), |x: f64, z: f64| {
    //     let (x_u64, z_u64): (u64, u64) = unsafe { std::mem::transmute((x, z)) };
    //     let y = surface_map.get(&(x_u64, z_u64)).unwrap().0;
    //     y
    // });
    let real_surface_points: PointSeries<_, _, Circle<_, _>, _> =
        PointSeries::new(SURFACES.real.clone(), 1_f64, &BLUE.mix(0.2));
    chart.draw_series(real_surface_points)?;

    let emu_surface_points: PointSeries<_, _, Circle<_, _>, _> =
        PointSeries::new(SURFACES.emu.clone(), 1_f64, &RED.mix(0.2));
    chart.draw_series(emu_surface_points)?;
    Ok(())
}
