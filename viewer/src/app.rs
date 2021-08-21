use anyhow::Result;
use cgmath::Point3;

use image::DynamicImage;
use sdl2::sys::SDL_SetWindowResizable;

use crate::{image_manager::ImageManager, presenter::Presenter, viewer::Viewer};

/// User interface of the image viewer app.
/// This struct prepare `Viewer`, `Presenter` and `ImageManager` to render images and widgets.
/// Users can add images and points via this struct.
pub struct App {
    viewer: Viewer,              // view
    presenter: Presenter,        // presenter
    image_manager: ImageManager, //model
}

impl App {
    /// Initialize sdl2 and opengl context and
    /// create `Viewer`, `Presente` and `ImageManager` instance.
    pub fn new(width: u32, height: u32) -> Result<App> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        {
            let gl_attr = video_subsystem.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 1);
            let (major, minor) = gl_attr.context_version();
            println!("OK : init OpenGL: version = {}.{}", major, minor);
        }
        let window = video_subsystem
            .window("SDL", width, height)
            .opengl()
            .position_centered()
            .build()
            .unwrap();
        unsafe {
            SDL_SetWindowResizable(window.raw(), sdl2::sys::SDL_bool::SDL_TRUE);
        }
        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

        let app = App {
            viewer: Viewer::new(sdl_context, video_subsystem, window, gl_context),
            presenter: Presenter::new(width, height),
            image_manager: ImageManager::new(),
        };
        Ok(app)
    }

    /// Start rendering images and widgets
    pub fn run(mut self) -> Result<()> {
        self.image_manager = self.image_manager.build();
        self.viewer.render(self.presenter, self.image_manager)
    }

    pub fn add_image(mut self, image: &DynamicImage, id: &str) -> Self {
        self.image_manager.add_image(image, id);
        self
    }

    pub fn add_images(mut self, images: &Vec<DynamicImage>, id_base: &str) -> Self {
        for i in 0..images.len() {
            let id = format!("{}_{}", id_base, i);
            self = self.add_image(images.get(i).unwrap(), &id);
        }
        self
    }

    pub fn add_point(
        mut self,
        image_id: &str,
        x: f32,
        y: f32,
        z: f32,
        r: f32,
        g: f32,
        b: f32,
    ) -> Self {
        self.image_manager.add_point(image_id, x, y, z, r, g, b);
        self
    }

    pub fn add_points(
        self,
        image_id: &str,
        points: Vec<Point3<f32>>,
        r: f32,
        g: f32,
        b: f32,
    ) -> Self {
        points.iter().fold(self, |app, pt| {
            app.add_point(image_id, pt.x, pt.y, pt.z, r, g, b)
        })
    }

    pub fn add_point_relation(
        mut self,
        lhs_key: &str,
        lx: f32,
        ly: f32,
        rhs_key: &str,
        rx: f32,
        ry: f32,
    ) -> Self {
        self.image_manager
            .add_point_relation(lhs_key, lx, ly, rhs_key, rx, ry);
        self
    }

    pub fn add_point_relations(
        mut self,
        points: &Vec<Vec<Point3<f32>>>,
        image_ids: &Vec<Vec<String>>,
    ) -> Self {
        assert_eq!(points.len(), image_ids.len());
        for (pts, ids) in points.iter().zip(image_ids) {
            assert_eq!(pts.len(), ids.len());
            for i in 0..(pts.len()) {
                for j in (i + 1)..(pts.len()) {
                    let lpt = pts.get(i).unwrap();
                    let rpt = pts.get(j).unwrap();
                    self = self.add_point_relation(
                        ids.get(i).unwrap(),
                        lpt.x,
                        lpt.y,
                        ids.get(j).unwrap(),
                        rpt.x,
                        rpt.y,
                    );
                }
            }
        }
        self
    }
}
