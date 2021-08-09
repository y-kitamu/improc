use std::path::Path;

use feat::keypoints::{fast::FASTCornerDetector, imgproc::gray, KeypointDetector};
use image::GenericImageView;

fn main() {
    let source_file = Path::new(file!());
    let project_root = source_file
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let image_file = project_root.join("data/sample_image/surface.png");
    println!("image file : {:?}", image_file);
    let mut image = image::open(image_file).unwrap();
    let gray = image::GrayImage::from_raw(
        image.width(),
        image.height(),
        gray(image.as_rgba8().unwrap()),
    )
    .unwrap();

    let fast = FASTCornerDetector::new(3, (50 * 50) as f32, 1);
    let feats = fast.detect(&gray, 0);

    let app = viewer::app::App::new(1280, 960).unwrap();
    image = image.flipv();
    println!("image_size = {}, {}", image.width(), image.height());
    let mut app = app.add_image(&image, "default");
    println!("feats num = {}", feats.len());
    app = feats.iter().fold(app, |ap, feat| {
        println!("(x, y) = {}, {}", feat.x(), feat.y());
        ap.add_point(
            "default",
            feat.x() / image.width() as f32 * 2.0 - 1.0,
            1.0 - feat.y() / image.height() as f32 * 2.0,
            1.0,
            1.0,
            0.0,
            0.0,
        )
    });
    app.run().unwrap();
}
