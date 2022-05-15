use std::{fs::create_dir, path::Path};

use improc::{feat::keypoints::KeyPoint, json_writer::ViewerWriter};

fn main() {
    let cur_file = Path::new(file!());
    let output_path = cur_file
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("output/sample.json");
    if !output_path.parent().unwrap().exists() {
        create_dir(output_path.parent().unwrap()).unwrap();
    }

    let mut writer = ViewerWriter::new(output_path.to_str().unwrap());
    let kpts = vec![
        KeyPoint::new(1, 2, 0.0f32, 1, 0.0f32),
        KeyPoint::new(2, 3, 0.0f32, 1, 0.0f32),
        KeyPoint::new(4, 5, 0.0f32, 1, 0.0f32),
        KeyPoint::new(6, 7, 0.0f32, 1, 0.0f32),
    ];
    writer.add_points(&kpts);
    writer.add_points(&kpts);
    writer.flush();
    println!("Output json : {}", output_path.to_str().unwrap());
}
