use std::{collections::HashMap, ptr};

use sdl2::event::Event;

use crate::{
    image_manager::ImageManager,
    shader::{self, Shader},
    vertex::{self, Vertex},
};

const DEFAULT_SHADER_KEY: &str = "default";

/// create frame buffer.
/// Return `frame_buffer_id`, `color_buffer_id`, `depth_buffer_id`
fn create_frame_buffer(width: u32, height: u32) -> (u32, u32, u32) {
    let mut frame_buffer_id: u32 = 0;
    let mut depth_buffer_id: u32 = 0;
    let mut color_buffer_id: u32 = 0;

    unsafe {
        // create frame buffer object
        gl::GenFramebuffers(1, &mut frame_buffer_id);
        gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buffer_id);

        // create color buffer (texture buffer)
        gl::GenTextures(1, &mut color_buffer_id);
        gl::BindTexture(gl::TEXTURE_2D, color_buffer_id);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            width as i32,
            height as i32,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            ptr::null(),
        );
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            color_buffer_id,
            0,
        );
        gl::BindTexture(gl::TEXTURE_2D, 0);

        // create depth buffer (render buffer)
        gl::GenRenderbuffers(1, &mut depth_buffer_id);
        gl::BindRenderbuffer(gl::RENDERBUFFER, depth_buffer_id);
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT24,
            width as i32,
            height as i32,
        );
        gl::FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER,
            depth_buffer_id,
        );

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("error: frame buffer is not complete");
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
    (frame_buffer_id, depth_buffer_id, color_buffer_id)
}

fn delete_fbo(frame_buffer_id: u32, depth_buffer_id: u32, color_buffer_id: u32) {
    unsafe {
        if 0 != frame_buffer_id {
            gl::DeleteFramebuffers(1, &frame_buffer_id);
        }
        if 0 != depth_buffer_id {
            gl::DeleteRenderbuffers(1, &depth_buffer_id);
        }
        if 0 != color_buffer_id {
            gl::DeleteTextures(1, &color_buffer_id);
        }
    }
}

// frame buffer object
pub struct Presenter {
    frame_buffer_id: u32,
    depth_buffer_id: u32,
    color_buffer_id: u32,
    fbo_width: u32,
    fbo_height: u32,
    fbo_vertex: Vertex,
    shader_map: HashMap<String, Shader>,
    current_shader_key: String,
}

impl Presenter {
    pub fn new(width: u32, height: u32) -> Presenter {
        let fbo_vertex = vertex::create_simple_vertex();
        let shader_map = shader::load_shaders();
        let current_shader_key = DEFAULT_SHADER_KEY.to_string();
        let (frame_buffer_id, depth_buffer_id, color_buffer_id) =
            create_frame_buffer(width, height);

        println!("current_shader_key = {}", current_shader_key);
        Presenter {
            frame_buffer_id,
            depth_buffer_id,
            color_buffer_id,
            fbo_width: width,
            fbo_height: height,
            fbo_vertex,
            shader_map,
            current_shader_key,
        }
    }

    pub fn process_event(&mut self, event: &Event) -> bool {
        let current_shader = self.shader_map.get_mut(&self.current_shader_key).unwrap();
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

    pub fn draw(&mut self, width: u32, height: u32, image_manager: &ImageManager) {
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
        }
        let image_texture_id = image_manager.get_current_texture_id();
        let (image_width, image_height) = image_manager.get_current_texture_image_size();
        let shader = self.shader_map.get_mut(&self.current_shader_key).unwrap();
        shader.adjust_aspect_ratio(image_width, image_height, width, height);
        let shader_id = shader.get_shader_id();

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buffer_id);
            // gl::Enable(gl::DEPTH_TEST);
            // gl::Disable(gl::BLEND);
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            // gl::Disable(gl::CULL_FACE);

            gl::Viewport(0, 0, width as i32, height as i32);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(shader_id);
            shader.set_uniform_variables();

            gl::BindTexture(gl::TEXTURE_2D, image_texture_id);
            self.fbo_vertex.draw();
            gl::BindTexture(gl::TEXTURE_2D, 0);

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn draw_imgui(&self, ui: &imgui::Ui) {}

    pub fn get_texture_id(&self) -> u32 {
        self.color_buffer_id
    }
}

impl Drop for Presenter {
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
