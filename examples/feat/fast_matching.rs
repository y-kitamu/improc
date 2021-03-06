//! FAST corner detector + brief特徴量 + brute force matchingのsample
use cgmath::Point3;
use clap::{AppSettings, Parser};
use image::{imageops::rotate180, DynamicImage, GenericImageView, GrayImage};
use nalgebra::Matrix2x3;
use std::{cmp::min, path::Path, time::Instant};

use improc::{
    feat::{
        descriptors::{
            brief::Brief, steered_brief::SteeredBrief, BriefDescriptor, Descriptor, Extractor,
        },
        keypoints::{fast::FASTCornerDetector, KeyPoint, KeypointDetector},
        matcher::{brute_force::BruteForceMathcer, Matcher},
    },
    imgproc::affine_transform,
    linalg::get_rotation_matrix,
    process_dynamic_image, timer,
};

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

fn compute_descriptor<T>(
    desc: &T,
    lhs_img: &GrayImage,
    lhs_kpts: &Vec<KeyPoint>,
    rhs_img: &GrayImage,
    rhs_kpts: &Vec<KeyPoint>,
) -> (
    Vec<Descriptor<BriefDescriptor>>,
    Vec<Descriptor<BriefDescriptor>>,
)
where
    T: Extractor<BriefDescriptor>,
{
    let descs0 = desc.compute(lhs_img, lhs_kpts);
    let descs1 = desc.compute(rhs_img, rhs_kpts);
    (descs0, descs1)
}

fn main() {
    env_logger::init();
    let opts: Opts = Opts::parse();
    let filename = match &opts.filename {
        Some(fname) => fname.clone(),
        None => Path::new(env!["CARGO_MANIFEST_DIR"])
            .join("data/sample_image/lena.png")
            .to_str()
            .unwrap()
            .to_string(),
    };
    let image = image::open(filename).unwrap();

    println!(
        "image size (width x height) = ({} x {}), color_type = {:?}",
        image.width(),
        image.height(),
        image.color(),
    );
    assert_eq!(
        image.as_bytes().len(),
        (image.width() * image.height() * 3) as usize
    );

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
    // let transformed = rotate180(&gray);

    let (all_feats0, all_feats1) = timer!("Fast Detector", {
        let fast = FASTCornerDetector::new(3, (50 * 50) as f32, 1, 1.0, true);
        let feats0 = fast.detect(&gray, 0);
        let feats1 = fast.detect(&transformed, 0);
        (feats0, feats1)
    });
    let feats0 = all_feats0[0..min(all_feats0.len(), opts.max_kpts)].to_vec();
    let feats1 = all_feats1[0..min(all_feats1.len(), opts.max_kpts)].to_vec();
    println!(
        "num feats0 = {}, num_feats1 = {}",
        all_feats0.len(),
        all_feats1.len()
    );

    let (descs0, descs1) = timer!("Brief descriptor", {
        if opts.descriptor == "brief" {
            let brief = Brief::new(31, 5, 256);
            compute_descriptor(&brief, &gray, &feats0, &transformed, &feats1)
        } else if opts.descriptor == "sbrief" {
            let brief = SteeredBrief::new(31, 5, 256, 12);
            compute_descriptor(&brief, &gray, &feats0, &transformed, &feats1)
        } else {
            (Vec::new(), Vec::new())
        }
    });

    let matches = timer!("Brute Force Matching", {
        let matcher = BruteForceMathcer::new(descs0, descs1, true);
        matcher.run()
    });

    let arrows0: Vec<(f32, f32, f32, f32)> = feats0
        .iter()
        .map(|kpt| (kpt.x(), kpt.y(), kpt.direction(), 1.0))
        .collect();
    let arrows1: Vec<(f32, f32, f32, f32)> = feats1
        .iter()
        .map(|kpt| (kpt.x(), kpt.y(), kpt.direction(), 1.0))
        .collect();
    let pts: Vec<Vec<Point3<f32>>> = vec![feats0, feats1]
        .iter()
        .map(|feats| feats.iter().map(|kpt| kpt.cgpt3d()).collect())
        .collect();

    let ms: Vec<Vec<(String, Point3<f32>)>> = matches
        .iter()
        .map(|m| {
            vec![
                ("gray".to_string(), m.matche.0.kpt.cgpt3d()),
                ("transformed".to_string(), m.matche.1.kpt.cgpt3d()),
            ]
        })
        .collect();

    let mps: Vec<Vec<Point3<f32>>> = ms.iter().map(|pair| vec![pair[0].1, pair[1].1]).collect();
    let ids: Vec<Vec<String>> = ms
        .iter()
        .map(|pair| vec![pair[0].0.clone(), pair[1].0.clone()])
        .collect();
    println!("num matches = {}", mps.len());
}
