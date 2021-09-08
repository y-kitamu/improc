use sdl2::{event::Event, mouse::MouseWheelDirection};

use crate::{
    model::image_manager::ImageManager,
    utility::{get_mouse_pos, scale_matrix},
};

use super::PresenterMode;

const SHADER_LIST: [&str; 1] = ["default"];

const DEFAULT_SHADER_KEY: &str = "default";

/// Presenter of MVP architecture.
/// This class holds frame buffer object for off-screen rendering.
pub struct DefaultPresenterMode {
    current_shader_key: String,
    current_image_key: String,
}

impl DefaultPresenterMode {
    pub const MODE_NAME: &'static str = "default";

    pub fn new() -> Self {
        let current_shader_key = DEFAULT_SHADER_KEY.to_string();
        let current_image_key = "".to_string();
        DefaultPresenterMode {
            current_shader_key,
            current_image_key,
        }
    }
}

impl Default for DefaultPresenterMode {
    fn default() -> Self {
        Self::new()
    }
}

impl PresenterMode for DefaultPresenterMode {
    fn get_mode_name(&self) -> &str {
        &Self::MODE_NAME
    }

    fn process_event(
        &self,
        event: &Event,
        fbo_width: u32,
        fbo_height: u32,
        mut image_manager: ImageManager,
    ) -> (ImageManager, bool) {
        let processed = match event {
            Event::MouseWheel { y, direction, .. } => {
                let (mx, my) = get_mouse_pos();
                let cx = mx as f32 / fbo_width as f32 * 2.0 - 1.0;
                let cy = (fbo_height as f32 - my as f32) / fbo_height as f32 * 2.0 - 1.0;
                let mut scale = 1.0f32 + *y as f32 / 10.0f32;
                if *direction == MouseWheelDirection::Flipped {
                    scale = 1.0f32 / scale;
                }
                image_manager.on_mouse_wheel(&self.current_image_key, cx, cy, scale);
                true
            }
            Event::MouseButtonDown { x, y, .. } => {
                // 左上(0, 0), 右下(width, height)の座標系を
                // 中心(0, 0), 左上(-1.0, 1.0), 右下(1.0, -1.0)の座標系に変換する
                let fx = *x as f32 / fbo_width as f32 * 2.0f32 - 1.0f32;
                let fy = 1.0f32 - *y as f32 / fbo_height as f32 * 2.0f32;
                image_manager.on_mouse_button_down(&self.current_image_key, fx, fy);
                true
            }
            Event::MouseButtonUp { .. } => {
                image_manager.on_mouse_button_up(&self.current_image_key);
                true
            }
            Event::MouseMotion { xrel, yrel, .. } => {
                let dx = *xrel as f32 / fbo_width as f32 * 2.0f32;
                let dy = -*yrel as f32 / fbo_height as f32 * 2.0f32;
                image_manager.on_mouse_motion_event(&self.current_image_key, dx, dy);
                true
            }
            _ => false,
        };
        (image_manager, processed)
    }

    fn draw(&mut self, width: u32, height: u32, mut image_manager: ImageManager) -> ImageManager {
        if self.current_image_key.len() == 0 {
            return image_manager;
        }

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
        image_manager.draw(&self.current_image_key, width, height);
        image_manager
    }

    fn draw_imgui(&self, ui: &imgui::Ui, image_manager: ImageManager) -> ImageManager {
        image_manager
    }
}
