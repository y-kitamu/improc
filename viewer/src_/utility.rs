use std::ops::Deref;

use cgmath::num_traits::ToPrimitive;
use image::{ImageBuffer, Pixel, RgbImage};
use sdl2::sys::SDL_GetMouseState;

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

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::One;

    #[test]
    fn test_convert_to_rgb() {
        let length = 10;
        let gray = image::GrayImage::from_fn(length, length, |x, y| image::Luma([(x + y) as u8]));
        let color = convert_to_rgb(&gray);
        assert_eq!(color.len() as u32, length * length * 3);

        for y in 0..length {
            for x in 0..length {
                let p = color.get_pixel(x, y);
                assert_eq!(p.0[0], (x + y) as u8);
                assert_eq!(p.0[0], p.0[1]);
                assert_eq!(p.0[1], p.0[2]);
            }
        }
    }

    #[test]
    fn test_scale_matrix() {
        let mat = Matrix4::one();
        let cx = 10.0;
        let cy = -10.0;
        let scale = 1.5;

        let dst = scale_matrix(&mat, cx, cy, scale);
        assert!((dst[0][0] - 1.5).abs() < 1e-5, "dst[0][1] = {}", dst[0][0]);
        assert!((dst[1][1] - 1.5).abs() < 1e-5, "dst[1][1] = {}", dst[1][1]);
        assert!((dst[3][0] - 0.0).abs() < 1e-5, "dst[3][0] = {}", dst[3][0]);
        assert!((dst[3][1] - 0.0).abs() < 1e-5, "dst[3][1] = {}", dst[3][1]);

        let dst = scale_matrix(&dst, 0.75, -0.3, 2.0);
        assert!((dst[0][0] - 3.0).abs() < 1e-5, "dst[0][1] = {}", dst[0][0]);
        assert!((dst[1][1] - 3.0).abs() < 1e-5, "dst[1][1] = {}", dst[1][1]);
        assert!((dst[3][0] + 0.75).abs() < 1e-5, "dst[3][0] = {}", dst[3][0]);
        assert!((dst[3][1] - 0.30).abs() < 1e-5, "dst[3][1] = {}", dst[3][1]);
    }
}
