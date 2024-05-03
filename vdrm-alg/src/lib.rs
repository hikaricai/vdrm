use geo::{EuclideanDistance, LineInterpolatePoint, LineIntersection};
use glam::Vec3Swizzles;
use std::collections::BTreeMap;

pub const W_PIXELS: usize = 64;
pub const H_PIXELS: usize = 32;
const CIRCLE_R: f32 = 1.;
pub const TOTAL_ANGLES: usize = 360;

type PixelColor = u32;
type PixelXY = (u32, u32);
pub type PixelSurface = Vec<(u32, u32, (u32, PixelColor))>;
pub type FloatSurface = Vec<(f32, f32, f32)>;
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct ScreenLineAddr {
    screen_idx: usize,
    addr: u32,
}
#[derive(Debug, Copy, Clone)]
struct ScreenLinePixels {
    pixels: [Option<PixelColor>; W_PIXELS],
}

lazy_static::lazy_static! {
    static ref SCREENS:[Screen; 3]  = {
        let u:(f32, f32) = (-2., 0.);
        let v:(f32, f32) = (-1., -1.);
        let w:(f32, f32) = (1., -1.);
        let x:(f32, f32) = (1. + 0.5_f32.sqrt(), 1. - 0.5_f32.sqrt());
        let y:(f32, f32) = (1. - 0.5_f32.sqrt(), 1. + 0.5_f32.sqrt());
        let z:(f32, f32) = (-1., 3.0_f32.sqrt());
        [Screen::new([v, w]), Screen::new([x, y]), Screen::new([z, u])]
    };
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct ScreenPixel {
    idx: usize,
    addr: u32,
    pixel: u32,
}

struct PixelZInfo {
    angle: u32,
    // 从底部向下增长
    value: f32,
    pixel: u32,
    screen_pixel: ScreenPixel,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ScreenLine {
    pub screen_idx: usize,
    pub addr: u32,
    pub pixels: [Option<PixelColor>; W_PIXELS],
}

pub type AngleMap = BTreeMap<u32, Vec<ScreenLine>>;

type PixelXYMap = BTreeMap<PixelXY, Vec<PixelZInfo>>;

struct Screen {
    xy_line: geo::Line<f32>,
}

impl Screen {
    fn new(xy: [(f32, f32); 2]) -> Self {
        let xy_line = geo::Line::new(xy[0], xy[1]);
        Self { xy_line }
    }
}

pub fn pixel_surface_to_float(pixel_surface: &PixelSurface) -> FloatSurface {
    pixel_surface
        .into_iter()
        .map(|&(pixel_x, pixel_y, (pixel_z, _color))| {
            let x = pixel_to_v(pixel_x);
            let y = pixel_to_v(pixel_y);
            let z = pixel_to_h(pixel_z);
            (x, y, z)
        })
        .collect()
}

fn pixel_to_v(p: u32) -> f32 {
    let point_size: f32 = 2. * CIRCLE_R / W_PIXELS as f32;
    p as f32 * point_size + 0.5 * point_size - CIRCLE_R
}

fn v_to_pixel(v: f32) -> Option<u32> {
    let point_size: f32 = 2. * CIRCLE_R / W_PIXELS as f32;
    let v = (v + CIRCLE_R) / point_size - 0.5;
    if v < 0. || v > 63. {
        return None;
    }
    Some(v as u32)
}

fn pixel_to_h(p: u32) -> f32 {
    let point_size: f32 = CIRCLE_R / H_PIXELS as f32;
    (p as f32) * point_size + 0.5 * point_size
}

fn h_to_pixel(h: f32) -> u32 {
    let point_size: f32 = CIRCLE_R / H_PIXELS as f32;
    (h / point_size - 0.5) as u32
}

fn angle_to_v(p: u32) -> f32 {
    ((p * 360 / TOTAL_ANGLES as u32) as f32).to_radians()
}

fn cacl_z_pixel(
    screens: &[Screen],
    mat: glam::Mat3A,
    angle: u32,
    x: u32,
    y: u32,
) -> Option<PixelZInfo> {
    let x = pixel_to_v(x);
    let y = pixel_to_v(y);
    let p1_view = glam::Vec3A::new(x, y, 0.);
    let p1 = mat * p1_view;
    if p1.z < -CIRCLE_R || p1.z > CIRCLE_R {
        return None;
    }
    let p2_view = glam::Vec3A::new(x, y, -3.);
    let p2 = mat * p2_view;

    let line_p1_p2 = geo::Line::new((p1.x, p1.y), (p2.x, p2.y));
    let mut intersection_info = None;
    for (idx, screen) in screens.iter().enumerate() {
        if let Some(LineIntersection::SinglePoint { intersection, .. }) =
            geo::line_intersection::line_intersection(screen.xy_line, line_p1_p2)
        {
            intersection_info = Some((intersection, idx));
            break;
        }
    }
    let Some((ps, screen_idx)) = intersection_info else {
        return None;
    };
    let px = screens[screen_idx].xy_line.start;
    let len_ps_px = px.euclidean_distance(&ps);
    let len_p1_px = px.euclidean_distance(&line_p1_p2.start);
    let view_h = CIRCLE_R * 2. - len_p1_px;
    let screen_pixel_h = p1.z;
    let addr = v_to_pixel(len_ps_px - CIRCLE_R)?;
    let pixel = v_to_pixel(screen_pixel_h)?;
    Some(PixelZInfo {
        angle,
        value: view_h,
        pixel: h_to_pixel(view_h),
        screen_pixel: ScreenPixel {
            idx: screen_idx,
            addr,
            pixel,
        },
    })
}

fn cacl_view_point(mat: glam::Mat3A, screen_idx: usize, addr: u32, pixel_z: u32) -> (f32, f32, f32) {
    let screen = &SCREENS[screen_idx];
    let len_start_s = pixel_to_v(addr) + CIRCLE_R;
    let fraction = len_start_s / (CIRCLE_R * 2.);
    let xy: geo::Coord<_> = screen.xy_line.line_interpolate_point(fraction).unwrap().into();
    let z = pixel_to_v(pixel_z);
    let p = glam::Vec3A::new(xy.x, xy.y, z);
    let p_view = mat * p;
    (p_view.x, p_view.y, p_view.z)
}

pub struct Codec {
    xy_map: PixelXYMap,
    mat_map: BTreeMap<u32, glam::Mat3A>,
}

impl Codec {
    pub fn new() -> Self {
        let mut xy_map = PixelXYMap::default();
        let mut mat_map = BTreeMap::new();
        for angle in 0..TOTAL_ANGLES as u32 {
            let angle_f = angle_to_v(angle);
            let sin = angle_f.sin();
            let cos = angle_f.cos();
            let sin2 = sin * sin;
            let cos2 = cos * cos;
            let sin_cos = sin * cos;
            let mat = glam::Mat3A::from_cols(
                glam::Vec3A::new(sin2, -sin_cos, -cos),
                glam::Vec3A::new(-sin_cos, cos2, -sin),
                glam::Vec3A::new(-cos, -sin, 0.),
            );
            mat_map.insert(angle, mat);
            for x in 0..W_PIXELS {
                for y in 0..W_PIXELS {
                    let (x, y) = (x as u32, y as u32);
                    let Some(z) = cacl_z_pixel(&SCREENS, mat, angle, x, y) else {
                        continue;
                    };
                    let entry = xy_map.entry((x, y)).or_default();
                    entry.push(z);
                }
            }
        }
        for z_info in xy_map.values_mut() {
            z_info.sort_by_key(|v| v.pixel);
        }
        Self { xy_map, mat_map }
    }

    pub fn encode(&self, pixel_surface: &PixelSurface, pixel_offset: i32) -> AngleMap {
        let mut angle_map: BTreeMap<u32, BTreeMap<ScreenLineAddr, ScreenLinePixels>> =
            BTreeMap::new();
        for &(x, y, (z, color)) in pixel_surface {
            let z_info_list = self.xy_map.get(&(x, y)).unwrap();
            let z_info_idx = match z_info_list.binary_search_by_key(&z, |v| v.pixel) {
                Ok(idx) => idx,
                Err(idx) => {
                    if idx == 0 {
                        idx
                    } else if idx >= z_info_list.len() {
                        idx - 1
                    } else {
                        let idx_l = idx - 1;
                        let l = z_info_list.get(idx_l).unwrap();
                        let r = z_info_list.get(idx).unwrap();
                        // let deta_l = z - l.pixel;
                        // let deta_r = r.pixel - z;
                        // if deta_l < deta_r
                        if z * 2 < l.pixel + r.pixel {
                            idx_l
                        } else {
                            idx
                        }
                    }
                }
            };
            let z_info = z_info_list.get(z_info_idx).or(z_info_list.last()).unwrap();
            let entry = angle_map.entry(z_info.angle).or_default();
            let addr = ScreenLineAddr {
                screen_idx: z_info.screen_pixel.idx,
                addr: z_info.screen_pixel.addr,
            };
            let line_pixels = entry.entry(addr).or_default();
            let pixel_idx = z_info.screen_pixel.pixel as usize;
            if let Some(c) = line_pixels.pixels.get_mut(pixel_idx) {
                *c = Some(color);
            } else {
                panic!("{x}, {y}, {z}, pixel_idx {pixel_idx}");
            }
        }
        angle_map
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.into_iter()
                        .map(|(k, mut v)| {
                            if pixel_offset > 0 {
                                let offset = pixel_offset as usize;
                                v.pixels.rotate_right(offset);
                                v.pixels.iter_mut().take(offset).for_each(|v| {
                                    v.take();
                                });
                            } else if pixel_offset < 0 {
                                let offset = (-pixel_offset) as usize;
                                v.pixels.rotate_left(offset);
                                v.pixels.iter_mut().rev().take(offset).for_each(|v| {
                                    v.take();
                                });
                            }
                            ScreenLine {
                                screen_idx: k.screen_idx,
                                addr: k.addr,
                                pixels: v.pixels,
                            }
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect()
    }
    pub fn decode(&self, angle: u32, lines: &[ScreenLine]) -> FloatSurface {
        let mut float_surface = FloatSurface::default();
        for ScreenLine {
            screen_idx,
            addr,
            pixels,
        } in lines
        {
            for (idx, pixel) in pixels.into_iter().enumerate() {
                let Some(_pixel) = pixel else { continue };
                let pixel_z = idx as u32;
                let mat = self.mat_map.get(&angle).unwrap();
                float_surface.push(cacl_view_point(*mat, *screen_idx, *addr, pixel_z));
            }
        }
        float_surface
    }
    pub fn decode_all(&self, angle_map: AngleMap) -> FloatSurface {
        let mut float_surface = FloatSurface::default();
        for (angle, lines) in angle_map {
            self.decode(angle, lines.as_slice())
        }
        float_surface
    }
}
