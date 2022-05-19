use std::path::Path;

use nalgebra as na;

use image::GenericImageView;
use improc::{
    feat::keypoints::{fast::FASTCornerDetector, KeypointDetector},
    imgproc::gray,
    json_writer::ViewerWriter,
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

    let cur_file = Path::new(file!());
    let output_path = cur_file
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("output/sample_feat.json");
    let mut writer = ViewerWriter::new(output_path.to_str().unwrap());
    writer.add_points(&feats, &na::Vector3::from_vec(vec![1.0, 0.0, 0.0]));
    let output_str = writer.flush().unwrap();
    println!("Output json : {}", output_path.to_str().unwrap());
    println!("{}", output_str);
}
