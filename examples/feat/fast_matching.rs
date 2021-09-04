//! FAST corner detector + brief特徴量 + brute force matchingのsample
use cgmath::Point3;
use clap::{AppSettings, Clap};
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
    linalg::get_rotation_matrix,
    process_dynamic_image, timer,
};

#[derive(Clap)]
#[clap(version = "1.0", author = "Y. Kitamu")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short, long)]
    filename: Option<String>,

    #[clap(long, default_value = "0.0")]
    dx: f32,

    #[clap(long, default_value = "0.0")]
    dy: f32,

    #[clap(long, default_value = "0.0")]
    rot_angle: f32,
}

fn get_affine_mat(image: &DynamicImage, opts: &Opts) -> Matrix2x3<f32> {
    let (width, height) = (image.width(), image.height());
    let mut mat = get_rotation_matrix(
        opts.rot_angle / 180.0f32 * std::f32::consts::PI,
        (width as f32 / 2.0, height as f32 / 2.0),
        1.0,
    );
    mat.m13 += opts.dx;
    mat.m23 += opts.dy;
    mat
}

fn main() {
    let opts: Opts = Opts::parse();
    let filename = match &opts.filename {
        Some(fname) => fname.clone(),
        None => Path::new(env!["CARGO_MANIFEST_DIR"])
            .join("data/sample_image/surface.png")
            .to_str()
            .unwrap()
            .to_string(),
    };
    let image = image::open(filename).unwrap();

    let gray = image::GrayImage::from_raw(
        image.width(),
        image.height(),
        process_dynamic_image!(&image, improc::imgproc::gray),
    )
    .unwrap();

    let affine_mat: Matrix2x3<f32> = get_affine_mat(&image, &opts);
    let transformed = image::GrayImage::from_raw(
        gray.width(),
        gray.height(),
        affine_transform(&gray, &affine_mat),
    )
    .unwrap();

    let (feats0, feats1) = timer!("Fast Detector", {
        let fast = FASTCornerDetector::new(3, (50 * 50) as f32, 1, true);
        let feats0 = fast.detect(&gray, 0);
        let feats1 = fast.detect(&transformed, 0);
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
        let descs1 = brief.compute(&transformed, &feats1);
        (descs0, descs1)
    });

    let matches = timer!("Brute Force Matching", {
        let matcher = BruteForceMathcer::new("gray", descs0, "transformed", descs1, true);
        matcher.run("gray", "transformed")
    });

    let pts: Vec<Vec<Point3<f32>>> = vec![feats0, feats1]
        .iter()
        .map(|feats| feats.iter().map(|kpt| kpt.cgpt3d()).collect())
        .collect();

    let ms: Vec<Vec<(String, Point3<f32>)>> = matches
        .iter()
        .map(|m| {
            m.matches
                .iter()
                .map(|(key, val)| (key.clone(), val.kpt.cgpt3d()))
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
        .add_image(&DynamicImage::ImageLuma8(transformed), "transformed")
        .add_points("gray", &pts[0], 1.0, 0.0, 0.0)
        .add_points("transformed", &pts[1], 1.0, 0.0, 0.0)
        .add_point_relations(&mps, &ids)
        .run()
        .unwrap();
}
