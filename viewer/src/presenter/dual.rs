use sdl2::event::Event;

use crate::image_manager::ImageManager;

use super::Presenter;

pub struct DualImagePresenter {}

impl Presenter for DualImagePresenter {
    fn get_texture_id(&self) -> u32 {
        0
    }

    fn process_event(&mut self, event: &Event) -> bool {
        false
    }

    fn draw(&mut self, width: u32, height: u32, image_manager: &ImageManager) {}

    fn draw_imgui(&mut self, ui: &imgui::Ui) {}
}
