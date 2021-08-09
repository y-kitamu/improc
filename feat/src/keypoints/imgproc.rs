use std::ops::Deref;

use image::{ColorType, ImageBuffer, Pixel};
use num_traits::ToPrimitive;

/// convert to gray scale.
pub fn gray<P, Container>(img: &ImageBuffer<P, Container>) -> Vec<u8>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: Deref<Target = [P::Subpixel]>,
{
    let x_stride = P::CHANNEL_COUNT as usize;
    assert!(x_stride == 3 || x_stride == 4);

    let (width, height) = (img.width() as usize, img.height() as usize);
    let y_stride = width * x_stride;
    let data = img.as_raw();
    let mut gray: Vec<u8> = Vec::with_capacity(width * height);
    let mut factor: Vec<f32> = vec![0.299, 0.587, 0.114];
    if P::COLOR_TYPE == ColorType::Bgr8 || P::COLOR_TYPE == ColorType::Bgra8 {
        factor = vec![factor[2], factor[1], factor[0]];
    }

    for y in 0..height {
        let off_y = y_stride * y;
        for x in 0..width {
            let off = off_y + x * x_stride;
            let val = (factor[0] * data[off].to_f32().unwrap()
                + factor[1] * data[off + 1].to_f32().unwrap()
                + factor[2] * data[off + 2].to_f32().unwrap()) as u8;
            gray.push(val);
        }
    }
    gray
}

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
    let y_stride = img.width() as usize * x_stride;

    for y in 0..height {
        for x in 0..width {
            let (fx, fy) = (x as f32 * x_scale, y as f32 * y_scale);
            let (dx, dy) = (fx.fract(), fy.fract());
            let (ix, iy) = (fx.floor() as usize, fy.floor() as usize);
            let off = iy * y_stride + ix * x_stride;
            for c in 0..x_stride {
                resized.push(
                    ((1.0f32 - dx) * (1.0f32 - dy) * data[off + c].to_f32().unwrap()
                        + dx * (1.0f32 - dy) * data[off + x_stride + c].to_f32().unwrap()
                        + (1.0f32 - dx) * dy * data[off + y_stride + c].to_f32().unwrap()
                        + dx * dy * data[off + y_stride + x_stride + c].to_f32().unwrap())
                        as u8,
                );
            }
        }
    }

    resized
}

#[cfg(test)]
mod tests {
    use super::{gray, resize};

    #[test]
    fn test_gray() {
        let length = 256;
        let test_image = image::RgbImage::from_fn(length, length, |x, y| {
            image::Rgb([((x + y) / 2) as u8, 0, 0])
        });
        let res = gray(&test_image);

        let data = test_image.as_raw();
        for y in 0..length {
            for x in 0..length {
                let off = ((y * length + x) * 3) as usize;
                assert_eq!(
                    res[(y * length + x) as usize],
                    (data[off] as f32 * 0.299) as u8
                );
            }
        }
    }

    #[test]
    fn test_resize() {
        let length: u32 = 256;
        let half = length / 2;
        let test_image = image::RgbImage::from_fn(length, length, |x, y| {
            image::Rgb([((x + y) / 2) as u8, 0, 0])
        });
        let res = resize(&test_image, half, half);
        assert_eq!(res.len(), (half * half * 3) as usize);

        for y in 0..half {
            for x in 0..half {
                assert_eq!(
                    res[((y * half + x) * 3) as usize],
                    (x + y) as u8,
                    "(x, y) = {}, {}",
                    x,
                    y
                );
            }
        }
    }
}
