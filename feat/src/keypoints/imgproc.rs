use std::ops::Deref;

use image::{ImageBuffer, Pixel};
use num_traits::ToPrimitive;

/// resize `img` to size (width, height).
pub fn resize<P, Container>(img: &ImageBuffer<P, Container>, width: u32, height: u32) -> Vec<u8>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: Deref<Target = [P::Subpixel]>,
{
    let (width, height) = (width as usize, height as usize);
    let x_stride = P::CHANNEL_COUNT as usize;
    let data = img.as_raw();
    let mut resized: Vec<u8> = Vec::with_capacity(width * height * x_stride);

    let x_scale = img.width() as f32 / width as f32;
    let y_scale = img.height() as f32 / height as f32;
    let y_stride = width * x_stride;
    for y in 0..height {
        let y_offset = y * y_stride;
        for x in 0..width {
            let (fx, fy) = (x as f32 * x_scale, y as f32 * y_scale);
            let (dx, dy) = (fx.fract(), fy.fract());
            let (ix, iy) = (fx.floor() as usize, fy.floor() as usize);
            let off = iy * y_stride + ix * x_stride;
            for c in 0..x_stride {
                resized[y_offset + x * x_stride + c] =
                    ((1.0f32 - dx) * (1.0f32 - dy) * data[off + c].to_f32().unwrap()
                        + dx * (1.0f32 - dy) * data[off + x_stride + c].to_f32().unwrap()
                        + (1.0f32 - dx) * dy * data[off + y_stride + c].to_f32().unwrap()
                        + dx * dy * data[off + y_stride + x_stride + c].to_f32().unwrap())
                        as u8;
            }
        }
    }

    resized
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_resize() {}
}
