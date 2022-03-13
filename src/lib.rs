pub mod ellipse;
pub mod feat;
pub mod imgproc;
pub mod linalg;
pub mod slam;
pub mod utility;

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
