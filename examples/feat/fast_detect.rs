use std::path::Path;

use image::GenericImageView;
use improc::{
    feat::keypoints::{fast::FASTCornerDetector, KeypointDetector},
    imgproc::gray,
};

fn main() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let image_file = project_root.join("data/sample_image/surface.png");
    // let image_file = project_root.join("data/sample_image/IMG_20210722_170349.jpg");
    println!("image file : {:?}", image_file);
    let mut image = image::open(image_file).unwrap();
    let gray = image::GrayImage::from_raw(
        image.width(),
        image.height(),
        gray(image.as_rgba8().unwrap()),
        // gray(image.as_rgb8().unwrap()),
    )
    .unwrap();

    let fast = FASTCornerDetector::new(3, (50 * 50) as f32, 1, 1.0, true);
    let feats = fast.detect(&gray, 0);
}
