use std::path::Path;

use anyhow::Result;
use cgmath::Point3;
use thiserror::Error;

use image::DynamicImage;
use log::warn;
use sdl2::sys::SDL_SetWindowResizable;

use crate::{model::image_manager::ImageManager, presenter::Presenter, view::viewer::Viewer};

#[derive(Error, Debug)]
enum AppError {
    #[error("failed to initialize sdl2 : {0}")]
    SdlInitError(String),
}

/// User interface of the image viewer app.
/// This struct prepare `Viewer`, `Presenter` and `ImageManager` to render images and widgets.
/// Users can add images and points via this struct.
pub struct App {
    viewer: Viewer,                  // view
    presenter: Presenter,            // presenter
    pub image_manager: ImageManager, //model
}

impl App {
    /// Initialize sdl2 and opengl context and
    /// create `Viewer`, `Presente` and `ImageManager` instance.
    /// # Example
    /// ```no_run
    /// use viewer::app::App;
    /// let app = App::new(1280u32, 1080u32).unwrap();
    /// ```
    pub fn new(width: u32, height: u32) -> Result<App> {
        let sdl_context = sdl2::init().map_err(|err| AppError::SdlInitError(err))?;
        let video_subsystem = sdl_context
            .video()
            .map_err(|err| AppError::SdlInitError(err))?;
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

    pub fn add_image_from_path(mut self, image_path: &Path, id: &str, vertical_flip: bool) -> Self {
        if let Err(_) = self.image_manager.load_image(image_path, id, vertical_flip) {
            warn!(
                "Failed to add image of path : {}",
                image_path.to_str().unwrap_or("None")
            );
        }
        self
    }

    pub fn add_images(mut self, images: &Vec<DynamicImage>, id_base: &str) -> Self {
        for i in 0..images.len() {
            let id = format!("{}_{}", id_base, i);
            self = self.add_image(images.get(i).unwrap(), &id);
        }
        self
    }

    /// Add point to a image of key = `image_id`.
    /// Argument `x` and `y` are treated as point on the image coordinate system.
    /// A value range of `z` is from -1.0 to 1.0.
    /// Argument `r`, `g` and `b` are pixel values range from 0.0 to 1.0.
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
