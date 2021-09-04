use std::ops::Deref;

use cgmath::num_traits::ToPrimitive;
use image::{GrayImage, ImageBuffer, Pixel, RgbImage};

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
