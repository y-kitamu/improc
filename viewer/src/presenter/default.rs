use std::collections::HashMap;

use imgui::im_str;
use sdl2::event::Event;

use crate::{
    image_manager::ImageManager,
    presenter::create_frame_buffer,
    shader::{self, Shader},
    vertex::{self, Vertex},
};

use super::{delete_fbo, Presenter};

const DEFAULT_SHADER_KEY: &str = "default";

/// Presenter of MVP architecture.
/// This class holds frame buffer object for off-screen rendering.
pub struct DefaultPresenter {
    frame_buffer_id: u32,
    depth_buffer_id: u32,
    color_buffer_id: u32,
    fbo_width: u32,
    fbo_height: u32,
    fbo_vertex: Vertex,
    shader_map: HashMap<String, Shader>,
    current_shader_key: String,
    points_shader: Shader,
}

impl DefaultPresenter {
    pub fn new(width: u32, height: u32) -> DefaultPresenter {
        let fbo_vertex = vertex::create_simple_vertex();
        let shader_map = shader::load_shaders();
        let current_shader_key = DEFAULT_SHADER_KEY.to_string();
        let (frame_buffer_id, depth_buffer_id, color_buffer_id) =
            create_frame_buffer(width, height);
        let points_shader = Shader::new("points");

        println!("current_shader_key = {}", current_shader_key);
        DefaultPresenter {
            frame_buffer_id,
            depth_buffer_id,
            color_buffer_id,
            fbo_width: width,
            fbo_height: height,
            fbo_vertex,
            shader_map,
            current_shader_key,
            points_shader,
        }
    }

    fn get_current_shader(&self) -> &Shader {
        self.shader_map.get_mut(&self.current_shader_key).unwrap()
    }
}

impl Presenter for DefaultPresenter {
    fn get_texture_id(&self) -> u32 {
        self.color_buffer_id
    }

    fn process_event(&mut self, event: &Event) -> bool {
        // let current_shader = self.shader_map.get_mut(&self.current_shader_key).unwrap();
        let current_shader = self.get_current_shader();
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
                let fx = *x as f32 / self.fbo_width as f32 * 2.0f32 - 1.0f32;
                let fy = 1.0f32 - *y as f32 / self.fbo_height as f32 * 2.0f32;
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
                let dx = *xrel as f32 / self.fbo_width as f32 * 2.0f32;
                let dy = -*yrel as f32 / self.fbo_height as f32 * 2.0f32;
                current_shader
                    .on_mouse_motion_event(timestamp, window_id, which, mousestate, x, y, dx, dy);
                true
            }
            _ => false,
        };
        processed
    }

    fn draw(&mut self, width: u32, height: u32, image_manager: &ImageManager) {
        if (width != self.fbo_width) || (height != self.fbo_height) {
            delete_fbo(
                self.frame_buffer_id,
                self.depth_buffer_id,
                self.color_buffer_id,
            );
            let (fbi, dbi, cbi) = create_frame_buffer(width, height);
            self.frame_buffer_id = fbi;
            self.depth_buffer_id = dbi;
            self.color_buffer_id = cbi;
            self.fbo_width = width;
            self.fbo_height = height;
        }

        let image_texture_id = image_manager.get_current_texture_id();
        let (image_width, image_height) = image_manager.get_current_texture_image_size();
        let shader = self.get_current_shader();
        shader.adjust_aspect_ratio(image_width, image_height, width, height);
        let shader_id = shader.get_shader_id();

        let points_vertex = image_manager.get_current_points_vertex();
        let points_shader_id = self.points_shader.get_shader_id();

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buffer_id);
            gl::Enable(gl::PROGRAM_POINT_SIZE);

            gl::Viewport(0, 0, width as i32, height as i32);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(shader_id);
            shader.set_uniform_variables(shader_id, false);

            gl::BindTexture(gl::TEXTURE_2D, image_texture_id);
            self.fbo_vertex.draw();
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
        let shader = self.get_current_shader();
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

impl Drop for DefaultPresenter {
    fn drop(&mut self) {
        delete_fbo(
            self.frame_buffer_id,
            self.depth_buffer_id,
            self.color_buffer_id,
        );
        self.frame_buffer_id = 0;
        self.depth_buffer_id = 0;
        self.color_buffer_id = 0;
    }
}
