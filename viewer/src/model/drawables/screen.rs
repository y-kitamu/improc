use std::{cell::Cell, ptr};

use crate::{
    shader::{screen_shader::ScreenShader, Shader, UniformVariable},
    Mat4,
};

use super::{create_simple_vertex, Drawable};

/// Frame bufferを使用した描画のためのstruct
pub struct Screen {
    vao: u32,
    vertex_num: u32,
    shader: Cell<Box<dyn Shader>>,
    frame_buffer_id: u32,
    depth_buffer_id: u32,
    color_buffer_id: u32,
    width: u32,
    height: u32,
    associated: Vec<Box<dyn Drawable>>,
}

impl Screen {
    pub fn new(width: u32, height: u32) -> Self {
        let (fbi, dbi, cbi) = create_frame_buffer(width, height);
        let (vao, _, vertex_num) = create_simple_vertex();
        let shader = ScreenShader::new();

        Screen {
            vao,
            vertex_num,
            shader: Cell::new(Box::new(shader)),
            frame_buffer_id: fbi,
            depth_buffer_id: dbi,
            color_buffer_id: cbi,
            width,
            height,
            associated: Vec::new(),
        }
    }

    pub fn update_window_size(&mut self, width: u32, height: u32) {
        if (width != self.width) || (height != self.height) {
            delete_fbo(
                self.frame_buffer_id,
                self.depth_buffer_id,
                self.color_buffer_id,
            );
            let (fbi, dbi, cbi) = create_frame_buffer(width, height);
            self.frame_buffer_id = fbi;
            self.depth_buffer_id = dbi;
            self.color_buffer_id = cbi;
            self.width = width;
            self.height = height;
        }
    }

    pub fn prepare(&mut self, width: u32, height: u32) {
        self.update_window_size(width, height);
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buffer_id);
            gl::Enable(gl::PROGRAM_POINT_SIZE);
            // gl::Enable(gl::BLEND);

            gl::Viewport(0, 0, width as i32, height as i32);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}

impl Drawable for Screen {
    fn get_drawable_type(&self) -> super::DrawableType {
        super::DrawableType::Screen
    }

    fn get_vertex_num(&self) -> u32 {
        self.vertex_num
    }

    fn get_draw_type(&self) -> gl::types::GLenum {
        gl::TRIANGLES
    }

    /// !!! Do not use !!!
    fn get_model_mat(&mut self) -> crate::Mat4 {
        self.shader.get_mut().get_model_mat().value.clone()
    }

    fn get_mut_shader(&mut self) -> &mut Box<dyn crate::shader::Shader> {
        self.shader.get_mut()
    }

    fn get_associated_drawables(&mut self) -> &Vec<Box<dyn Drawable>> {
        &self.associated
    }

    fn get_mut_associated_drawables(&mut self) -> &mut Vec<Box<dyn Drawable>> {
        &mut self.associated
    }

    fn is_draw(&self) -> bool {
        true
    }

    fn set_is_draw(&mut self, _flag: bool) {}

    fn get_vao(&self) -> u32 {
        self.vao
    }

    fn get_texture_id(&self) -> u32 {
        self.color_buffer_id
    }

    fn build(&mut self) {}

    fn draw(&mut self, _view_mat: &UniformVariable<Mat4>, _proj_mat: &UniformVariable<Mat4>) {
        let shader_id = self.shader.get_mut().get_id();
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // gl::Enable(gl::DEPTH_TEST);
            // gl::Disable(gl::BLEND);
            // gl::Enable(gl::BLEND);
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            // gl::Disable(gl::CULL_FACE);

            gl::Viewport(0, 0, self.width as i32, self.height as i32);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(shader_id);
            gl::BindTexture(gl::TEXTURE_2D, self.color_buffer_id);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_num as i32);
            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::UseProgram(0);
        }
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui) {
        self.get_mut_shader().draw_imgui(ui);
    }

    fn update(
        &mut self,
        _view_mat: &crate::shader::UniformVariable<crate::Mat4>,
        _proj_mat: &crate::shader::UniformVariable<crate::Mat4>,
    ) {
    }
}

impl Drop for Screen {
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
