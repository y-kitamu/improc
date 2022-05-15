// independent from other module
pub mod linalg;
pub mod optimizer;
pub mod utility;

// depend on the other module
pub mod ellipse;
pub mod epipolar;
pub mod feat;
pub mod imgproc;
pub mod json_writer;
pub mod sfm;
pub mod slam;

use nalgebra as na;

#[macro_export]
macro_rules! process_dynamic_image {
    ($e:expr, $i:expr) => {
        match $e {
            image::DynamicImage::ImageLuma8(img) => $i(&img),
            image::DynamicImage::ImageLumaA8(img) => $i(&img),
            image::DynamicImage::ImageRgb8(img) => $i(&img),
            image::DynamicImage::ImageRgba8(img) => $i(&img),
            image::DynamicImage::ImageBgr8(img) => $i(&img),
            image::DynamicImage::ImageBgra8(img) => $i(&img),
            image::DynamicImage::ImageLuma16(img) => $i(&img),
            image::DynamicImage::ImageLumaA16(img) => $i(&img),
            image::DynamicImage::ImageRgb16(img) => $i(&img),
            image::DynamicImage::ImageRgba16(img) => $i(&img),
        }
    };
}

pub trait PrintDebug {
    fn print(&self);
}

impl<T: na::Scalar + std::fmt::Display, R: na::Dim, C: na::Dim, S: na::RawStorage<T, R, C>>
    PrintDebug for na::Matrix<T, R, C, S>
{
    fn print(&self) {
        (0..self.nrows()).for_each(|row_idx| {
            let row: Vec<T> = self.row(row_idx).iter().cloned().collect();
            println!(
                "{}",
                row.as_slice()
                    .iter()
                    .fold(String::from(""), |acc, val| format!("{}{:.3}, ", acc, val))
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print() {
        let mat = na::Matrix3::new(1.0, 1.111111, 2.000, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0);
        mat.print();
    }
}
