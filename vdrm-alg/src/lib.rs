use geo::{ClosestPoint, EuclideanDistance, EuclideanLength, LineInterpolatePoint};
use std::collections::BTreeMap;

pub const W_PIXELS: usize = 192;
pub const H_PIXELS: usize = 96;
const CIRCLE_R: f32 = 1.;
const POINT_SIZE: f32 = 2. * CIRCLE_R / W_PIXELS as f32;
pub const TOTAL_ANGLES: usize = W_PIXELS * 2 * 314 / 100;
// pub const TOTAL_ANGLES: usize = 360;

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

impl Default for ScreenLinePixels {
    fn default() -> Self {
        Self {
            pixels: [None; W_PIXELS],
        }
    }
}

pub const SCREEN_OFFSET: f32 = std::f32::consts::SQRT_2;

lazy_static::lazy_static! {
    static ref SCREENS:[Screen; 3]  = {
        // let u:(f32, f32) = (-2., 0.);
        // let v:(f32, f32) = (-1., -1.);
        // let w:(f32, f32) = (1., -1.);
        // let x:(f32, f32) = (1. + 0.5_f32.sqrt(), 1. - 0.5_f32.sqrt());
        // let y:(f32, f32) = (1. - 0.5_f32.sqrt(), 1. + 0.5_f32.sqrt());
        // let z:(f32, f32) = (-1., 3.0_f32.sqrt());
        // [Screen::new([v, w]), Screen::new([x, y]), Screen::new([z, u])]
        let l = 1. + SCREEN_OFFSET;
        let depth = 2f32;
        let a:(f32, f32) = (0., l);
        let rad = std::f32::consts::PI / 9.;
        //let rad = 0f32;
        // let b:(f32, f32) = (0. - 1., 1. + 3f32.sqrt());
        // let b:(f32, f32) = (0. + 1., 1. + 3f32.sqrt());
        let b:(f32, f32) = (0. - depth* rad.sin(), l + depth * rad.cos());
        let x_offset = (-b.0 / 4.0);
        let offset_rad = (x_offset / l).asin();
        let y_offset = offset_rad.cos() * l - l;
        let a = (a.0 + x_offset, a.1 + y_offset);
        let b = (b.0 + x_offset, b.1 + y_offset);

        let rad = std::f32::consts::FRAC_PI_8;
        let l1 = l + depth;
        let c = (-l * rad.sin(), l * rad.cos());
        // let d = (-l1 * rad.sin(), l1 * rad.cos());
        let rad = std::f32::consts::PI / 9.;
        let d = (c.0 + depth * rad.sin(), c.1 + depth * rad.cos());
        let e = (l * rad.sin(), l * rad.cos());
        // let f = (l1 * rad.sin(), l1 * rad.cos());
        let f = (e.0 - depth * rad.sin(), e.1 + depth * rad.cos());
        [Screen::new([a, b]), Screen::new([c, d]), Screen::new([e, f])]
    };
}

pub fn screens() -> &'static [Screen] {
    &*SCREENS
}
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct ScreenPixel {
    idx: usize,
    addr: u32,
    pixel: u32,
}

#[derive(Clone, Copy, Debug)]
struct PixelZInfo {
    angle: u32,
    pixel: u32,
    is_borrowed: bool,
    screen_pixel: ScreenPixel,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ScreenLine {
    pub screen_idx: usize,
    pub addr: u32,
    pub pixels: [Option<PixelColor>; W_PIXELS],
}

pub type AngleMap = BTreeMap<u32, [Vec<ScreenLine>; 3]>;

type PixelZInfoList = [Option<PixelZInfo>; H_PIXELS];
type PixelXYArr = Vec<Vec<PixelZInfoList>>;

#[derive(Copy, Clone)]
pub struct Screen {
    pub xy_line: geo::Line<f32>,
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
    let point_size: f32 = POINT_SIZE;
    p as f32 * point_size + 0.5 * point_size - CIRCLE_R
}

fn v_to_pixel(v: f32) -> Option<u32> {
    let point_size: f32 = POINT_SIZE;
    let v = (v + CIRCLE_R) / point_size - 0.5;
    if v < -POINT_SIZE {
        return None;
    }
    let mut v = v as u32;
    if v > W_PIXELS as u32 {
        return None;
    }
    if v == W_PIXELS as u32 {
        v -= 1;
    }
    Some(v)
}

fn pixel_to_h(p: u32) -> f32 {
    (p as f32) * POINT_SIZE
}

fn h_to_pixel(mut h: f32) -> Option<u32> {
    let point_size: f32 = POINT_SIZE;
    // fix z offset
    h += POINT_SIZE;
    if h < -POINT_SIZE {
        return None;
    }
    let mut p = (h / point_size) as u32;
    if p > H_PIXELS as u32 {
        return None;
    }
    if p == H_PIXELS as u32 {
        p -= 1;
    }
    Some(p)
}

fn v3_2_pixel(x: f32, y: f32, z: f32) -> Option<(u32, u32, u32)> {
    let x = v_to_pixel(x)?;
    let y = v_to_pixel(y)?;
    let z = h_to_pixel(z)?;
    // fix offset
    // if z + 1 < H_PIXELS as u32 {
    //     z += 1;
    // }
    Some((x, y, z))
}

pub fn angle_to_v(p: u32) -> f32 {
    (p as f32 * 360. / TOTAL_ANGLES as f32).to_radians()
}

fn cacl_view_point(
    mat: glam::Mat3A,
    screen_idx: usize,
    addr: u32,
    pixel_z: u32,
) -> ((f32, f32, f32), (f32, f32, f32)) {
    let screen = &SCREENS[screen_idx];
    let len_start_s = pixel_to_v(addr) + CIRCLE_R;
    let fraction = len_start_s / (CIRCLE_R * 2.);
    let xy: geo::Coord<_> = screen
        .xy_line
        .line_interpolate_point(fraction)
        .unwrap()
        .into();
    let z = pixel_to_v(pixel_z) - SCREEN_OFFSET;
    let p = glam::Vec3A::new(xy.x, xy.y, z);
    let p_view = mat * p;
    ((p_view.x, p_view.y, p_view.z), (p.x, p.y, p.z))
}

fn closest_len(line: &geo::Line<f32>, p: &geo::Point<f32>) -> f32 {
    let close_p = match line.closest_point(p) {
        geo::Closest::Intersection(point) => point,
        geo::Closest::SinglePoint(point) => point,
        geo::Closest::Indeterminate => unreachable!(),
    };
    close_p.euclidean_distance(p)
}

pub struct Codec {
    xy_arrs: [PixelXYArr; 3],
    mat_map: BTreeMap<u32, glam::Mat3A>,
}

impl Codec {
    // TODO map screens to image and fill tthe xy_arr
    pub fn new(angle_range: std::ops::Range<usize>) -> Self {
        let mut xy_arrs = [PixelXYArr::new(), PixelXYArr::new(), PixelXYArr::new()];
        for _x in 0..W_PIXELS {
            let mut line = vec![];
            line.extend(
                std::iter::repeat(())
                    .take(W_PIXELS)
                    .map(|_| [None; H_PIXELS]),
            );
            xy_arrs[0].push(line.clone());
            xy_arrs[1].push(line.clone());
            xy_arrs[2].push(line);
        }
        let mut mat_map = BTreeMap::new();
        let screen_metas: Vec<_> = SCREENS
            .iter()
            .map(|screen| {
                let start = screen.xy_line.start;
                let end = screen.xy_line.end;
                let p_o = glam::Vec3A::new(start.x, start.y, -1. - SCREEN_OFFSET);
                // let p_a = glam::Vec3A::new(start.x, start.y, 1.);
                // let p_b = glam::Vec3A::new(end.x, end.y, -1.);

                let v_oa = glam::Vec3A::new(0., 0., POINT_SIZE);
                let v_ob = geo::Line::new(start, end);
                let v_ob = v_ob
                    .line_interpolate_point(POINT_SIZE / v_ob.euclidean_length())
                    .unwrap()
                    - start.into();
                let v_ob = glam::Vec3A::new(v_ob.x(), v_ob.y(), 0.);
                log::info!("p_o {p_o:?} v_oa {v_oa:?} v_ob {v_ob:?}");
                (screen, p_o, v_oa, v_ob)
            })
            .collect();
        for angle in 0..TOTAL_ANGLES {
            let angle = angle as u32;
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

            let center = glam::Vec3A::new(0., 0. + SCREEN_OFFSET, -1.5 - SCREEN_OFFSET);
            let center = mat * center;
            let center_xy = geo::Point::new(center.x, center.y);

            for (screen_idx, (screen, p_o, v_oa, v_ob)) in
                screen_metas.clone().into_iter().enumerate()
            {
                let closest_len = closest_len(&screen.xy_line, &center_xy);
                // log::info!("angle {angle} closest_len {closest_len}");
                if closest_len > 2f32.sqrt() {
                    continue;
                }

                let p_o = mat * p_o;
                let v_oa = mat * v_oa;
                let v_ob = mat * v_ob;
                if angle == 100 {
                    log::info!("p_o {p_o:?} v_oa {v_oa:?} v_ob {v_ob:?}");
                }

                for i in 0..W_PIXELS {
                    for j in 0..W_PIXELS {
                        let p = p_o + v_oa * (i as f32) + v_ob * (j as f32);
                        if angle >= 99 && angle <= 101 {
                            // log::info!("i {i} j {j} p {p}");
                        }
                        let pz = -p.z - 1.0 - SCREEN_OFFSET;
                        let px = p.x;
                        let py = p.y - SCREEN_OFFSET;
                        let Some((x, y, z)) = v3_2_pixel(px, py, pz) else {
                            continue;
                        };
                        if angle >= 99 && angle <= 101 {
                            // log::info!("x {x} y {y} z {z}");
                        }
                        let z_point = PixelZInfo {
                            angle,
                            pixel: z,
                            is_borrowed: false,
                            screen_pixel: ScreenPixel {
                                idx: screen_idx,
                                addr: j as u32,
                                pixel: i as u32,
                            },
                        };
                        xy_arrs[screen_idx][x as usize][y as usize][z as usize] = Some(z_point);
                    }
                }
            }
        }
        // TODO borrow value from other coloums
        // not good

        // TODO no clone
        // let xy_arr_cl = xy_arr.clone();
        // let borrow = |x: usize, y: usize, z: usize, z_info: &mut Option<PixelZInfo>| match xy_arr_cl
        //     .get(x)
        //     .and_then(|v| v.get(y))
        //     .and_then(|v| v[z])
        //     .as_ref()
        // {
        //     Some(v) => {
        //         if v.is_borrowed {
        //             return false;
        //         }
        //         let mut v = v.clone();
        //         v.is_borrowed = true;
        //         *z_info = Some(v);
        //         return true;
        //     }
        //     None => return false,
        // };

        // for (x, y_arr) in xy_arr.iter_mut().enumerate() {
        //     for (y, z_arr) in y_arr.iter_mut().enumerate() {
        //         for (z, z_info) in z_arr.iter_mut().enumerate() {
        //             if z_info.is_some() {
        //                 continue;
        //             }

        //             if borrow(x + 1, y, z, z_info) {
        //                 continue;
        //             }
        //             if borrow(x, y + 1, z, z_info) {
        //                 continue;
        //             }
        //             if x > 1 {
        //                 if borrow(x - 1, y, z, z_info) {
        //                     continue;
        //                 }
        //             }
        //             if y > 1 {
        //                 if borrow(x, y - 1, z, z_info) {
        //                     continue;
        //                 }
        //             }
        //         }
        //     }
        // }
        Self { xy_arrs, mat_map }
    }

    pub fn encode(
        &self,
        pixel_surface: &PixelSurface,
        pixel_offset: i32,
        optimze_speed_for_mbi5264: bool,
    ) -> AngleMap {
        let mut angle_map: BTreeMap<u32, [BTreeMap<ScreenLineAddr, ScreenLinePixels>; 3]> =
            BTreeMap::new();
        for &(x, y, (z, color)) in pixel_surface {
            for screen_idx in 0..3usize {
                let z_info_list = &self.xy_arrs[screen_idx][x as usize][y as usize];
                // fix z offset
                // TODO find the reason for offset
                // let z = if z > 1 { z - 1 } else { z };
                let Some(z_info) = z_info_list.get(z as usize).and_then(|v| *v) else {
                    continue;
                };
                let entry = angle_map.entry(z_info.angle).or_default();
                let addr = ScreenLineAddr {
                    screen_idx: z_info.screen_pixel.idx,
                    addr: z_info.screen_pixel.addr,
                };
                let line_pixels = entry[screen_idx].entry(addr).or_default();
                let pixel_idx = z_info.screen_pixel.pixel as usize;
                if let Some(c) = line_pixels.pixels.get_mut(pixel_idx) {
                    *c = Some(color);
                } else {
                    panic!("{x}, {y}, {z}, pixel_idx {pixel_idx}");
                }
            }
        }
        angle_map
            .into_iter()
            .map(|(k, addr_maps)| {
                (
                    k,
                    addr_maps.map(|addr_map| {
                        parse_addr_map(addr_map, pixel_offset, optimze_speed_for_mbi5264)
                    }),
                )
            })
            .collect()
    }
    pub fn decode(&self, angle: u32, lines: &[ScreenLine]) -> (FloatSurface, FloatSurface) {
        let mut view_surface = FloatSurface::default();
        let mut led_surface = FloatSurface::default();
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
                let (view, led) = cacl_view_point(*mat, *screen_idx, *addr, pixel_z);
                view_surface.push(view);
                led_surface.push(led);
            }
        }
        (view_surface, led_surface)
    }
    pub fn decode_all(&self, angle_map: AngleMap) -> (FloatSurface, FloatSurface) {
        let mut view_surface = FloatSurface::default();
        let mut led_surface = FloatSurface::default();
        for (angle, lines_arr) in angle_map {
            for lines in lines_arr {
                let (view, led) = self.decode(angle, lines.as_slice());
                view_surface.extend(view);
                led_surface.extend(led);
            }
        }
        (view_surface, led_surface)
    }
}

fn parse_addr_map(
    mut addr_map: BTreeMap<ScreenLineAddr, ScreenLinePixels>,
    pixel_offset: i32,
    optimze_speed_for_mbi5264: bool,
) -> Vec<ScreenLine> {
    let mut pixels_info: [Option<([u8; 4], ScreenLineAddr)>; W_PIXELS] = [None; W_PIXELS];
    for (addr, line) in addr_map.iter_mut() {
        for (color, pixel_info) in line.pixels.iter_mut().zip(&mut pixels_info) {
            let Some(rgbh) = color.as_ref() else {
                continue;
            };
            let [r, g, b, _a] = rgbh.to_ne_bytes();
            match pixel_info {
                Some(_rgbh) => {
                    *color = None;
                }
                None => {
                    *pixel_info = Some(([r, g, b, addr.addr as u8], *addr));
                }
            }
        }
    }
    if optimze_speed_for_mbi5264 {
        for i in 0..64usize {
            let region0 = i;
            let region1 = i + 64;
            let region2 = i + 128;
            let regions = [region1, region0, region2];
            let mut non_empty_h: Option<u8> = None;
            for region in regions {
                let Some(pixel_info) = pixels_info[region] else {
                    continue;
                };
                if pixel_info.0[..3] == [0; 3] {
                    continue;
                }
                non_empty_h = Some(pixel_info.0[3]);
                break;
            }
            let Some(non_empty_h) = non_empty_h else {
                continue;
            };
            let non_empty_h_mod = non_empty_h % 16;
            for region in regions {
                let Some((rgbh, line_addr)) = &pixels_info[region] else {
                    continue;
                };
                let h = rgbh[3];
                let h_mod = h % 16;
                if h_mod == non_empty_h_mod {
                    continue;
                }
                let mut deta = non_empty_h_mod as i16 - h_mod as i16;
                if deta.abs() > 8 {
                    let try_deta = if deta > 0 { deta - 16 } else { deta + 16 };
                    let try_h = h as i16 + try_deta as i16;
                    if try_h >= 0 && try_h <= 143 {
                        deta = try_deta;
                    }
                }
                let new_h = (h as i16 + deta) as u8;
                let line_pixels = addr_map.get_mut(&line_addr).unwrap();
                let pixel = line_pixels.pixels[region].take().unwrap();
                let mut line_addr = *line_addr;
                line_addr.addr = new_h as u32;
                let entry = addr_map.entry(line_addr).or_default();
                entry.pixels[region] = Some(pixel);
            }
        }
    }

    addr_map
        .into_iter()
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
        .collect::<Vec<_>>()
}
