use clap::{App, Arg};
use std::{any::Any, path::Path};

fn main() {
    let matches = App::new("My Super Program")
        .version("1.0")
        .arg(
            Arg::with_name("filename")
                .short("f")
                .long("filename")
                .required(false),
        )
        .get_matches();

    let project_root = Path::new(env!["CARGO_MANIFEST_DIR"]);
    let default_image_file = project_root.join("data/sample_image/surface.png");
    let filename = match matches.value_of("filename") {
        Some(f) => f,
        None => default_image_file.to_str().unwrap(),
    };

    println!("filename = {}", filename);

    let mut image = image::open(filename).unwrap();
    // let gray = image::GrayImage::from_raw(
    //     image.width(),
    //     image.height(),
    //     gray(image.)
    // )
}
