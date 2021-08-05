use viewer;

fn main() {
    env_logger::init();
    println!("Hello world!");
    viewer::hello_from_viewer();

    let app = viewer::app::App::new(1280, 960).unwrap();

    let image_file = "/home/kitamura/work/Projects/improc/data/sample_image/surface.png";
    let mut image = image::open(image_file).unwrap();
    image = image.flipv();

    app.add_image(&image, "default")
        .add_point("default", 0.0f32, 0.0f32, 0.6f32, 1.0f32, 0.0f32, 0.0f32)
        .add_point("default", 1.0f32, 0.0f32, 0.6f32, 1.0f32, 0.0f32, 0.0f32)
        .add_point("default", 0.5f32, 0.5f32, 0.6f32, 1.0f32, 1.0f32, 0.0f32)
        .run()
        .unwrap();
}
