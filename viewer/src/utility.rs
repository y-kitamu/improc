use std::ops::Deref;

use cgmath::num_traits::ToPrimitive;
use image::{ImageBuffer, Pixel, RgbImage};
use sdl2::sys::{SDL_GL_GetCurrentWindow, SDL_GetMouseState, SDL_GetWindowSize};

use crate::Matrix4;

/// convert gray to color (3d) image.
pub fn convert_to_rgb<P, Container>(img: &ImageBuffer<P, Container>) -> RgbImage
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: Deref<Target = [P::Subpixel]>,
{
    let (width, height) = (img.width(), img.height());
    let data = img.as_raw();
    let x_stride = P::CHANNEL_COUNT as usize;
    RgbImage::from_fn(width, height, |x, y| {
        let val = data[(y * width + x) as usize * x_stride].to_u8().unwrap();
        image::Rgb([val, val, val])
    })
}

/// mouse positionを取得する
/// `normalized` == trueの場合、normalized texture coordinate (-1.0 ~ 1.0)
pub fn get_mouse_pos() -> (i32, i32) {
    let mut x = 0;
    let mut y = 0;
    let _: u32 = unsafe { SDL_GetMouseState(&mut x, &mut y) };
    (x, y)
}

pub fn get_window_size() -> (f32, f32) {
    let mut x = 0;
    let mut y = 0;
    unsafe { SDL_GetWindowSize(SDL_GL_GetCurrentWindow(), &mut x, &mut y) };
    (x as f32, y as f32)
}

pub fn scale_matrix(model_mat: &Matrix4, cx: f32, cy: f32, scale: f32) -> Matrix4 {
    let mut dst = model_mat.clone();
    let mut x = (cx - model_mat[3][0]) / (model_mat[0][0] + 1e-9);
    let mut y = (cy - model_mat[3][1]) / (model_mat[1][1] + 1e-9);
    if x.abs() > 1.0 || y.abs() > 1.0 {
        x = 0.0;
        y = 0.0;
    }
    let dx = (1.0 - scale) * x;
    let dy = (1.0 - scale) * y;
    dst[3][0] += dst[0][0] * dx;
    dst[3][1] += dst[1][1] * dy;
    dst[0][0] *= scale;
    dst[1][1] *= scale;
    dst
}
