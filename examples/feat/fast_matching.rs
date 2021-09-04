//! FAST corner detector + brief特徴量 + brute force matchingのsample
use cgmath::Point3;
use clap::{App, Arg};
use image::{DynamicImage, GenericImageView};
use nalgebra::{matrix, Matrix2x3};
use std::{path::Path, time::Instant};

use improc::{
    feat::{
        descriptors::{brief::Brief, Extractor},
        keypoints::{fast::FASTCornerDetector, KeypointDetector},
        matcher::{brute_force::BruteForceMathcer, Matcher},
    },
    imgproc::affine_transform,
    process_dynamic_image, timer,
};

fn main() {
    let matches = App::new("My Super Program")
        .version("1.0")
        .arg(
            Arg::with_name("filename")
                .short("f")
                .long("filename")
                .value_name("FILE")
                .required(false),
        )
        .get_matches();
    println!("{:?}", matches);

    let project_root = Path::new(env!["CARGO_MANIFEST_DIR"]);
    let default_image_file = project_root.join("data/sample_image/surface.png");
    let filename = matches
        .value_of("filename")
        .unwrap_or(default_image_file.to_str().unwrap());
    println!("filename = {}", filename);

    let image = image::open(filename).unwrap();

    let gray = image::GrayImage::from_raw(
        image.width(),
        image.height(),
        process_dynamic_image!(&image, improc::imgproc::gray),
    )
    .unwrap();

    let affine_mat: Matrix2x3<f32> = matrix![
        1.0f32, 0.0f32, 10.0f32;
        0.0f32, 1.0f32, 10.0f32;
    ];
    let shift = image::GrayImage::from_raw(
        gray.width(),
        gray.height(),
        affine_transform(&gray, &affine_mat),
    )
    .unwrap();

    let (feats0, feats1) = timer!("Fast Detector", {
        let fast = FASTCornerDetector::new(3, (50 * 50) as f32, 1, true);
        let feats0 = fast.detect(&gray, 0);
        let feats1 = fast.detect(&shift, 0);
        (feats0, feats1)
    });
    println!(
        "num feats0 = {}, num_feats1 = {}",
        feats0.len(),
        feats1.len()
    );

    let (descs0, descs1) = timer!("Brief descriptor", {
        let brief = Brief::new(31, 5, 256);
        let descs0 = brief.compute(&gray, &feats0);
        let descs1 = brief.compute(&shift, &feats1);
        (descs0, descs1)
    });

    let matches = timer!("Brute Force Matching", {
        let matcher = BruteForceMathcer::new("gray", descs0, "shift", descs1, true);
        matcher.run("gray", "shift")
    });

    let pts: Vec<Vec<Point3<f32>>> = vec![feats0, feats1]
        .iter()
        .map(|feats| {
            feats
                .iter()
                .map(|kpt| Point3::<f32>::new(kpt.x(), kpt.y(), 1.0))
                .collect()
        })
        .collect();

    let ms: Vec<Vec<(String, Point3<f32>)>> = matches
        .iter()
        .map(|m| {
            m.matches
                .iter()
                .map(|(key, val)| {
                    (
                        key.clone(),
                        Point3::<f32>::new(val.kpt.x(), val.kpt.y(), 1.0),
                    )
                })
                .collect()
        })
        .collect();
    let mps: Vec<Vec<Point3<f32>>> = ms.iter().map(|pair| vec![pair[0].1, pair[1].1]).collect();
    let ids: Vec<Vec<String>> = ms
        .iter()
        .map(|pair| vec![pair[0].0.clone(), pair[1].0.clone()])
        .collect();
    println!("num matches = {}", mps.len());
    let app = viewer::app::App::new(1280, 960).unwrap();
    app.add_image(&image, "color")
        .add_image(&DynamicImage::ImageLuma8(gray), "gray")
        .add_image(&DynamicImage::ImageLuma8(shift), "shift")
        .add_points("gray", &pts[0], 1.0, 1.0, 0.0)
        .add_points("shift", &pts[1], 1.0, 1.0, 0.0)
        .add_point_relations(&mps, &ids)
        .run()
        .unwrap();
}
