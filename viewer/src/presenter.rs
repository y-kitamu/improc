use std::{collections::HashMap, ptr};

use c_str_macro::c_str;
use cgmath::{perspective, Matrix4, Point3, Vector3};

use crate::{
    image_manager::ImageManager,
    shader::{self, Shader},
    vertex::{self, Vertex},
};

const DEFAULT_SHADER_KEY: &str = "default";

// frame buffer object
pub struct Presenter {
    frame_buffer_id: u32,
    depth_buffer_id: u32,
    color_buffer_id: u32,
    fbo_vertex: Vertex,
    shader_map: HashMap<String, Shader>,
    current_shader_key: String,
}

impl Presenter {
    pub fn new(width: u32, height: u32) -> Presenter {
        let fbo_vertex = vertex::create_simple_vertex();
        let shader_map = shader::load_shaders();
        let current_shader_key = DEFAULT_SHADER_KEY.to_string();

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

        println!("current_shader_key = {}", current_shader_key);
        Presenter {
            frame_buffer_id,
            depth_buffer_id,
            color_buffer_id,
            fbo_vertex,
            shader_map,
            current_shader_key,
        }
    }

    pub fn draw(&self, width: u32, height: u32, image_manager: &ImageManager) {
        let image_texture_id = image_manager.get_current_texture_id();
        let shader = self.shader_map.get(&self.current_shader_key).unwrap();
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

            gl::BindTexture(gl::TEXTURE_2D, image_texture_id);
            self.fbo_vertex.draw();
            gl::BindTexture(gl::TEXTURE_2D, 0);

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn get_texture_id(&self) -> u32 {
        self.color_buffer_id
    }
}

impl Drop for Presenter {
    fn drop(&mut self) {
        unsafe {
            if 0 != self.frame_buffer_id {
                gl::DeleteFramebuffers(1, &self.frame_buffer_id);
                self.frame_buffer_id = 0;
            }
            if 0 != self.color_buffer_id {
                gl::DeleteTextures(1, &self.color_buffer_id);
                self.color_buffer_id = 0;
            }
            if 0 != self.depth_buffer_id {
                gl::DeleteRenderbuffers(1, &self.depth_buffer_id);
                self.depth_buffer_id = 0;
            }
        }
    }
}
