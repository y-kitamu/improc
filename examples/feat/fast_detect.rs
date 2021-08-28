use std::path::Path;

use feat::keypoints::{fast::FASTCornerDetector, imgproc::gray, KeypointDetector};
use image::GenericImageView;

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

    let fast = FASTCornerDetector::new(3, (50 * 50) as f32, 1, true);
    let feats = fast.detect(&gray, 0);

    let app = viewer::app::App::new(1280, 960).unwrap();
    image = image.flipv();
    println!("image_size = {}, {}", image.width(), image.height());
    let mut app = app.add_image(&image, "default").add_image(&image, "other");
    println!("feats num = {}", feats.len());
    app = feats.iter().fold(app, |ap, feat| {
        // println!("(x, y) = {}, {}", feat.x(), feat.y());
        ap.add_point("default", feat.x(), feat.y(), 1.0, 1.0, 0.0, 0.0)
            .add_point("other", feat.x(), feat.y(), 1.0, 1.0, 0.0, 0.0)
            .add_point_relation("default", feat.x(), feat.y(), "other", feat.x(), feat.y())
    });
    app.run().unwrap();
}
