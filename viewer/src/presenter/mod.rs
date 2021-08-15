mod default;
mod dual;

use std::{collections::HashMap, ptr};

use sdl2::event::Event;

use crate::{
    image_manager::ImageManager,
    vertex::{self, Vertex},
};

use self::{default::DefaultPresenterMode, dual::DualImagePresenter};

pub trait PresenterMode {
    fn get_mode_name(&self) -> &str;

    /// process user action event, and return whether event is processed.
    fn process_event(&mut self, event: &Event, fbo_width: u32, fbo_height: u32) -> bool;

    /// draw images and points to frame buffer object for off screen rendering
    fn draw(
        &mut self,
        width: u32,
        height: u32,
        image_manager: &ImageManager,
        presenter: &Presenter,
    );

    /// draw imgui object to screen (not frame buffer object)
    fn draw_imgui(&mut self, ui: &imgui::Ui);
}

pub struct Presenter {
    modes: HashMap<String, Box<dyn PresenterMode>>,
    current_modes_key: String,
    frame_buffer_id: u32,
    depth_buffer_id: u32,
    color_buffer_id: u32,
    fbo_width: u32,
    fbo_height: u32,
    fbo_vertex: Vertex,
}

impl Presenter {
    const MODE_NAME: &'static str = "Presenter";

    pub fn new(width: u32, height: u32) -> Self {
        let fbo_vertex = vertex::create_simple_vertex();
        let (frame_buffer_id, depth_buffer_id, color_buffer_id) =
            create_frame_buffer(width, height);
        let mut presenter = Presenter {
            modes: HashMap::new(),
            current_modes_key: DefaultPresenterMode::MODE_NAME.to_string(),
            frame_buffer_id,
            depth_buffer_id,
            color_buffer_id,
            fbo_width: width,
            fbo_height: height,
            fbo_vertex,
        };

        let default_mode = Box::new(DefaultPresenterMode::new());
        presenter
            .modes
            .insert(default_mode.get_mode_name().to_string(), default_mode);

        let dual_mode = Box::new(DualImagePresenter::new());
        presenter
            .modes
            .insert(dual_mode.get_mode_name().to_string(), dual_mode);

        presenter
    }

    pub fn get_texture_id(&self) -> u32 {
        self.color_buffer_id
    }

    pub fn get_fbo_size(&self) -> (u32, u32) {
        (self.fbo_width, self.fbo_height)
    }

    pub fn get_fbo_vertex(&self) -> &Vertex {
        &self.fbo_vertex
    }

    pub fn get_frame_buffer_id(&self) -> u32 {
        self.frame_buffer_id
    }

    pub fn update_window_size(&mut self, width: u32, height: u32) {
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
    }
}

impl PresenterMode for Presenter {
    fn get_mode_name(&self) -> &str {
        Self::MODE_NAME
    }

    fn process_event(&mut self, event: &Event, fbo_width: u32, fbo_height: u32) -> bool {
        let current_mode = self.modes.get_mut(&self.current_modes_key).unwrap();
        current_mode.process_event(event, fbo_width, fbo_height)
    }

    fn draw(
        &mut self,
        width: u32,
        height: u32,
        image_manager: &ImageManager,
        presente: &Presenter,
    ) {
        self.update_window_size(width, height);
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui) {}
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
