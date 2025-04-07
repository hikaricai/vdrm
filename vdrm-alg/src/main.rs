use std::fmt::Display;

fn gen_pyramid_surface() -> vdrm_alg::PixelSurface {
    let mut pixel_surface = vdrm_alg::PixelSurface::new();
    let gray: u8 = 0xff;
    let r = vdrm_alg::W_PIXELS as i32 / 2;
    for x in 0..vdrm_alg::W_PIXELS as u32 {
        for y in 0..vdrm_alg::W_PIXELS as u32 {
            let x_i32 = x as i32 - r;
            let y_i32 = y as i32 - r;
            let h = x_i32.abs() + y_i32.abs();
            if h >= vdrm_alg::H_PIXELS as i32 {
                continue;
            }
            let z = vdrm_alg::H_PIXELS as u32 - 1 - h as u32;
            let color = match (x_i32 >= 0, y_i32 >= 0) {
                (true, true) => u32::from_ne_bytes([gray, gray, gray, 0]),
                (false, true) => u32::from_ne_bytes([gray, 0, 0, 0]),
                (false, false) => u32::from_ne_bytes([0, gray, 0, 0]),
                (true, false) => u32::from_ne_bytes([gray, 0, gray, 0]),
            };
            pixel_surface.push((x, y, (z, color)));
        }
    }
    pixel_surface
}

pub type RGBH = [u8; 4];
pub const IMG_HEIGHT: usize = 192;
pub const IMG_WIDTH: usize = IMG_HEIGHT / 4;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct AngleImage {
    pub angle: u32,
    // rgbh
    pub coloum: [RGBH; IMG_HEIGHT],
}

impl AngleImage {
    pub const fn new(angle: u32) -> Self {
        Self {
            angle,
            coloum: [[0; 4]; IMG_HEIGHT],
        }
    }
}

impl Display for AngleImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rgbh in self.coloum {
            let h = rgbh[3];
            write!(f, "{h:02}")?;
        }
        write!(f, "\n")
    }
}

fn main() {
    let codec = vdrm_alg::Codec::new(0..400);
    let pyramid = gen_pyramid_surface();
    let map = codec.encode(&pyramid, 0, true);
    let mut angle_list = vec![];
    for (angle, line) in map {
        let mut img = AngleImage::new(angle);
        let mut pixels: [Option<[u8; 4]>; IMG_HEIGHT] = [None; IMG_HEIGHT];
        for p in line {
            for (color, pixel) in p.pixels.iter().zip(&mut pixels) {
                let Some(color) = color else {
                    continue;
                };
                let [r, g, b, _a] = color.to_ne_bytes();
                match pixel {
                    Some(rgbh) => {
                        let h = rgbh[3];
                        if p.addr < h as u32 {
                            *rgbh = [r, g, b, p.addr as u8];
                        }
                    }
                    None => {
                        *pixel = Some([r, g, b, p.addr as u8]);
                    }
                }
            }
        }
        for (c, p) in img.coloum.iter_mut().zip(pixels) {
            if let Some(p) = p {
                *c = p;
            }
        }
        print!("{} {}", img.angle, img);
        angle_list.push(img);
    }
    let buf = unsafe {
        std::slice::from_raw_parts(
            angle_list.as_ptr() as *const u8,
            angle_list.len() * std::mem::size_of::<AngleImage>(),
        )
    };
}
