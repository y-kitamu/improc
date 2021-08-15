//! Image viewer app.
//! The program structure of this app is based on the MVP architecture.
//! `image_manager` module is Model of MVP,
//! `viewer` module is View of MVP, and
//! `presenter` module is Presenter of MVP.
//! `app` module is user interface.
//! `shader` module prepare and render glsl shader.

pub mod app;
mod image_manager;
mod presenter;
mod shader;
mod vertex;
mod viewer;
