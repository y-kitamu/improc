use anyhow::Result;
use thiserror::Error;

use crate::{
    model::{drawables::Drawable, viewer_model::ViewerModel, Model},
    presenter::{presenter::ViewerPresenter, Presenter},
    view::{viewer::Viewer, View},
};

#[derive(Error, Debug)]
enum AppError {
    #[error("failed to initialize sdl2 : {0}")]
    SdlInitError(String),
}

/// User interface of the image viewer app.
/// This struct prepare `Viewer`, `Presenter` and `ImageManager` to render images and widgets.
/// Users can add images and points via this struct.
pub struct App {
    model: Box<dyn Model>,
    view: Box<dyn View>,
}

impl App {
    /// create `Model` and `View` traits instances.
    /// # Example
    /// ```no_run
    /// ```
    pub fn new(width: u32, height: u32) -> Result<App> {
        let app = App {
            model: ViewerModel::new(),
            view: Viewer::new(width, height),
        };
        Ok(app)
    }

    /// Start rendering images and widgets
    pub fn run(self) -> Result<()> {
        let mut presenter = ViewerPresenter::new(self.model, self.view);
        presenter.render()
    }

    pub fn add_drawable(mut self, drawable: Box<dyn Drawable>) -> Self {
        self.model.add_drawable(drawable);
        self
    }
}
