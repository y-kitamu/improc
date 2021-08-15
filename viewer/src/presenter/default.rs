use std::collections::HashMap;

use imgui::im_str;
use sdl2::event::Event;

use crate::{
    image_manager::ImageManager,
    shader::{self, Shader},
};

use super::{Presenter, PresenterMode};

const SHADER_LIST: [&str; 1] = ["default"];
const POINTS_SHADER_LIST: [&str; 1] = ["points"];

const DEFAULT_SHADER_KEY: &str = "default";
const DEFAULT_POINTS_SHADER_KEY: &str = "points";

/// Presenter of MVP architecture.
/// This class holds frame buffer object for off-screen rendering.
pub struct DefaultPresenterMode {
    shader_map: HashMap<String, Shader>,
    points_shader_map: HashMap<String, Shader>,
    current_shader_key: String,
    current_image_key: String,
    current_points_shader_key: String,
}

impl DefaultPresenterMode {
    pub const MODE_NAME: &'static str = "default";

    pub fn new() -> Self {
        let shader_map = shader::load_shaders(&SHADER_LIST.to_vec());
        let points_shader_map = shader::load_shaders(&POINTS_SHADER_LIST.to_vec());
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

impl PresenterMode for DefaultPresenterMode {
    fn get_mode_name(&self) -> &str {
        &Self::MODE_NAME
    }

    fn process_event(&mut self, event: &Event, fbo_width: u32, fbo_height: u32) -> bool {
        let current_shader = self.shader_map.get_mut(&self.current_shader_key).unwrap();
        // let current_shader = self.get_current_shader();
        let processed = match event {
            Event::MouseWheel {
                timestamp,
                window_id,
                which,
                x,
                y,
                direction,
            } => {
                current_shader.on_mouse_wheel_event(timestamp, window_id, which, x, y, direction);
                true
            }
            Event::MouseButtonDown {
                timestamp,
                window_id,
                which,
                mouse_btn,
                clicks,
                x,
                y,
            } => {
                // 左上(0, 0), 右下(width, height)の座標系を
                // 中心(0, 0), 左上(-1.0, 1.0), 右下(1.0, -1.0)の座標系に変換する
                let fx = *x as f32 / fbo_width as f32 * 2.0f32 - 1.0f32;
                let fy = 1.0f32 - *y as f32 / fbo_height as f32 * 2.0f32;
                current_shader
                    .on_mouse_button_down(timestamp, window_id, which, mouse_btn, clicks, fx, fy);
                true
            }
            Event::MouseButtonUp {
                timestamp,
                window_id,
                which,
                mouse_btn,
                clicks,
                x,
                y,
            } => {
                current_shader
                    .on_mouse_button_up(timestamp, window_id, which, mouse_btn, clicks, x, y);
                true
            }
            Event::MouseMotion {
                timestamp,
                window_id,
                which,
                mousestate,
                x,
                y,
                xrel,
                yrel,
            } => {
                let dx = *xrel as f32 / fbo_width as f32 * 2.0f32;
                let dy = -*yrel as f32 / fbo_height as f32 * 2.0f32;
                current_shader
                    .on_mouse_motion_event(timestamp, window_id, which, mousestate, x, y, dx, dy);
                true
            }
            _ => false,
        };
        processed
    }

    fn draw(
        &mut self,
        width: u32,
        height: u32,
        image_manager: &ImageManager,
        presenter: &Presenter,
    ) {
        if self.current_image_key.len() == 0 {
            return;
        }
        let image_texture_id = image_manager.get_texture_id(&self.current_image_key);
        let (image_width, image_height) =
            image_manager.get_texture_image_size(&self.current_image_key);

        let shader = self.shader_map.get_mut(&self.current_shader_key).unwrap();
        shader.adjust_aspect_ratio(image_width, image_height, width, height);
        let shader_id = shader.get_shader_id();

        let points_vertex = image_manager.get_points_vertex(&self.current_image_key);
        let points_shader_id = self
            .points_shader_map
            .get(&self.current_points_shader_key)
            .unwrap()
            .get_shader_id();

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, presenter.get_frame_buffer_id());
            gl::Enable(gl::PROGRAM_POINT_SIZE);

            gl::Viewport(0, 0, width as i32, height as i32);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(shader_id);
            shader.set_uniform_variables(shader_id, false);

            gl::BindTexture(gl::TEXTURE_2D, image_texture_id);
            presenter.get_fbo_vertex().draw();
            gl::BindTexture(gl::TEXTURE_2D, 0);

            if let Some(pts_vtx) = points_vertex {
                gl::UseProgram(points_shader_id);
                shader.set_uniform_variables(points_shader_id, true);
                pts_vtx.draw_points();
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui) {
        let shader = self.shader_map.get_mut(&self.current_shader_key).unwrap();
        imgui::Window::new(im_str!("Parameters"))
            .size([200.0, 250.0], imgui::Condition::FirstUseEver)
            .position([700.0, 10.0], imgui::Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("Image parameter"));
                ui.separator();
                ui.separator();
                ui.text(im_str!("Point parameter"));
                imgui::Slider::new(im_str!("Point size"))
                    .range(1.0..=100.0)
                    .build(&ui, &mut shader.point_size.value);
                ui.separator();
            });
    }
}
