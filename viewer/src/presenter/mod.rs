mod default;
mod dual;

use std::ptr;

use sdl2::event::Event;

use crate::image_manager::ImageManager;

trait Presenter {
    /// process user action event, and return whether event is processed.
    fn process_event(&mut self, event: &Event) -> bool;

    /// draw images and points to frame buffer object for off screen rendering
    fn draw(&mut self, width: u32, height: u32, image_manager: &ImageManager);

    /// draw imgui object to screen (not frame buffer object)
    fn draw_imgui(&mut self, ui: &imgui::Ui);

    fn get_texture_id(&self) -> u32;
}

enum PresenterModes {
    Default(default::DefaultPresenter),
    Dual(dual::DualImagePresenter),
}

// TODO: use macro
impl Presenter for PresenterModes {
    fn process_event(&mut self, event: &Event) -> bool {
        match self {
            Self::Default(presenter) => presenter.process_event(event),
            Self::Dual(presenter) => presenter.process_event(event),
        }
    }

    fn draw(&mut self, width: u32, height: u32, image_manager: &ImageManager) {
        match self {
            Self::Default(presenter) => presenter.draw(width, height, image_manager),
            Self::Dual(presenter) => presenter.draw(width, height, image_manager),
        }
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui) {
        match self {
            Self::Default(presenter) => presenter.draw_imgui(ui),
            Self::Dual(presenter) => presenter.draw_imgui(ui),
        }
    }

    fn get_texture_id(&self) -> u32 {
        match self {
            Self::Default(presenter) => presenter.get_texture_id(),
            Self::Dual(presenter) => presenter.get_texture_id(),
        }
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
