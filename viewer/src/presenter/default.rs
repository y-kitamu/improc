use std::collections::HashMap;

use imgui::im_str;
use sdl2::{event::Event, mouse::MouseWheelDirection};

use crate::{
    model::image_manager::ImageManager,
    shader::{image_shader::ImageShader, point_shader::PointShader},
    utility::{get_mouse_pos, scale_matrix},
};

use super::PresenterMode;

const SHADER_LIST: [&str; 1] = ["default"];
const POINTS_SHADER_LIST: [&str; 1] = ["points"];

const DEFAULT_SHADER_KEY: &str = "default";
const DEFAULT_POINTS_SHADER_KEY: &str = "points";

/// Presenter of MVP architecture.
/// This class holds frame buffer object for off-screen rendering.
pub struct DefaultPresenterMode {
    shader_map: HashMap<String, ImageShader>,
    points_shader_map: HashMap<String, PointShader>,
    current_shader_key: String,
    current_image_key: String,
    current_points_shader_key: String,
}

impl DefaultPresenterMode {
    pub const MODE_NAME: &'static str = "default";

    pub fn new() -> Self {
        let shader_map = load_shaders!(SHADER_LIST, ImageShader);
        let points_shader_map = load_shaders!(POINTS_SHADER_LIST, PointShader);
        let current_shader_key = DEFAULT_SHADER_KEY.to_string();
        let current_image_key = "".to_string();
        let current_points_shader_key = DEFAULT_POINTS_SHADER_KEY.to_string();
        DefaultPresenterMode {
            shader_map,
            points_shader_map,
            current_shader_key,
            current_image_key,
            current_points_shader_key,
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

    fn process_event(&mut self, event: &Event, fbo_width: u32, fbo_height: u32) -> bool {
        let current_shader = self.shader_map.get_mut(&self.current_shader_key).unwrap();
        // let current_shader = self.get_current_shader();
        let processed = match event {
            Event::MouseWheel { y, direction, .. } => {
                let (mx, my) = get_mouse_pos();
                let cx = mx as f32 / fbo_width as f32 * 2.0 - 1.0;
                let cy = (fbo_height as f32 - my as f32) / fbo_height as f32 * 2.0 - 1.0;
                let mut scale = 1.0f32 + *y as f32 / 10.0f32;
                if *direction == MouseWheelDirection::Flipped {
                    scale = 1.0f32 / scale;
                }
                current_shader.model_mat.value =
                    scale_matrix(&current_shader.model_mat.value, cx, cy, scale);
                true
            }
            Event::MouseButtonDown { x, y, .. } => {
                // 左上(0, 0), 右下(width, height)の座標系を
                // 中心(0, 0), 左上(-1.0, 1.0), 右下(1.0, -1.0)の座標系に変換する
                let fx = *x as f32 / fbo_width as f32 * 2.0f32 - 1.0f32;
                let fy = 1.0f32 - *y as f32 / fbo_height as f32 * 2.0f32;
                current_shader.on_mouse_button_down(fx, fy);
                true
            }
            Event::MouseButtonUp { .. } => {
                current_shader.on_mouse_button_up();
                true
            }
            Event::MouseMotion { xrel, yrel, .. } => {
                let dx = *xrel as f32 / fbo_width as f32 * 2.0f32;
                let dy = -*yrel as f32 / fbo_height as f32 * 2.0f32;
                current_shader.on_mouse_motion_event(dx, dy);
                true
            }
            _ => false,
        };
        processed
    }

    fn draw(&mut self, width: u32, height: u32, image_manager: &ImageManager) {
        if self.current_image_key.len() == 0 {
            return;
        }
        let (image_width, image_height) =
            image_manager.get_texture_image_size(&self.current_image_key);

        let shader = self.shader_map.get_mut(&self.current_shader_key).unwrap();
        shader.adjust_aspect_ratio(image_width, image_height, width, height);

        let pts_shader = self
            .points_shader_map
            .get(&self.current_points_shader_key)
            .unwrap();
        unsafe {
            shader.set_uniform_variables();
            image_manager.draw_image(&self.current_image_key);
            gl::UseProgram(0);

            pts_shader.set_uniform_variables(&shader);
            image_manager.draw_points(&self.current_image_key);
            gl::UseProgram(0);
        }
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui, image_manager: &ImageManager) {
        imgui::Window::new(im_str!("Parameters"))
            .size([200.0, 250.0], imgui::Condition::FirstUseEver)
            .position([700.0, 10.0], imgui::Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("Image parameter"));
                ui.separator();

                for key in image_manager.get_image_keys() {
                    let mut flag = self.current_image_key == *key;
                    if ui.radio_button(&im_str!("{}", key), &mut flag, true) {
                        self.current_image_key = key.clone();
                    }
                }

                ui.separator();
                ui.text(im_str!("Point parameter"));
                let shader = self
                    .points_shader_map
                    .get_mut(&self.current_points_shader_key)
                    .unwrap();
                imgui::Slider::new(im_str!("Point size"))
                    .range(1.0..=100.0)
                    .build(&ui, &mut shader.point_size.value);
                ui.separator();
            });
    }
}
