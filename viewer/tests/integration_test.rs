use cgmath::Point3;
use viewer;

#[test]
fn test_app() {
    let app = viewer::app::App::new(10u32, 10u32);
    if app.is_err() {
        return;
    }
    let app = app.unwrap();

    let (w, h) = (10, 5);
    let key = "sample";
    let sample_img = image::DynamicImage::new_rgb8(w, h);
    let base_key = "base";
    let sample_imgs = vec![
        image::DynamicImage::new_bgr8(w, h),
        image::DynamicImage::new_luma8(w, h),
    ];
    let mut app = app
        .add_image(&sample_img, key)
        .add_images(&sample_imgs, base_key)
        .add_point(key, w as f32, h as f32, 1.0, 0.0, 1.0, 1.0)
        .add_points(
            "base_1",
            &vec![
                Point3::<f32>::new(w as f32, h as f32, 1.0),
                Point3::<f32>::new(h as f32, w as f32, 1.0),
            ],
            1.0,
            0.0,
            1.0,
        )
        .add_point_relation(key, w as f32, h as f32, "base_1", h as f32, w as f32);
    app.image_manager = app.image_manager.build();
}
