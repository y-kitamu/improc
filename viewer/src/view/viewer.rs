use std::{
    fmt::{self, Display},
    time::Duration,
};

use anyhow::Result;
use imgui::im_str;
use log::info;
use sdl2::{event::Event, keyboard::Keycode, Sdl, VideoSubsystem};

use crate::{
    define_gl_primitive, draw,
    model::{create_simple_vertex, GLPrimitive},
    shader::image_shader::ImageShader,
};
use crate::{model::image_manager::ImageManager, presenter::Presenter};

#[derive(Debug)]
struct ViewerError(String);

impl Display for ViewerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ViewerError : {}", self.0)
    }
}

impl std::error::Error for ViewerError {}

/// View of MVP architecture.
pub struct Viewer {
    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    window: sdl2::video::Window,
    _gl_context: sdl2::video::GLContext,
    screen_shader: ImageShader,
    vao: Option<u32>,
    vbo: Option<u32>,
    vertex_num: i32,
}

define_gl_primitive!(Viewer);

impl Viewer {
    pub fn new(
        sdl_context: Sdl,
        video_subsystem: VideoSubsystem,
        window: sdl2::video::Window,
        gl_context: sdl2::video::GLContext,
    ) -> Viewer {
        let (vao, vbo, vertex_num) = create_simple_vertex();
        let screen_shader = ImageShader::new("screen");

        let viewer = Viewer {
            sdl_context,
            video_subsystem,
            window,
            _gl_context: gl_context,
            screen_shader,
            vao: Some(vao),
            vbo: Some(vbo),
            vertex_num,
        };

        info!("OK : Init Viewer.");
        viewer
    }

    pub fn render(self, mut presenter: Presenter, mut image_manager: ImageManager) -> Result<()> {
        let mut imgui_context = imgui::Context::create();
        imgui_context.set_ini_filename(None);

        let mut imgui_sdl2_context = imgui_sdl2::ImguiSdl2::new(&mut imgui_context, &self.window);
        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui_context, |s| {
            self.video_subsystem.gl_get_proc_address(s) as _
        });

        let mut event_pump = self.sdl_context.event_pump().unwrap();
        'running: loop {
            for event in event_pump.poll_iter() {
                imgui_sdl2_context.handle_event(&mut imgui_context, &event);
                if imgui_sdl2_context.ignore_event(&event) {
                    continue;
                }
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {
                        let (im, _) = presenter.process_event(&event, image_manager);
                        image_manager = im;
                    }
                }
            }
            // draw image to fbo
            let (width, height) = self.window.size();
            image_manager = presenter.draw(width, height, image_manager);

            // draw fbo to screen
            self.draw(presenter.get_texture_id());

            // draw imgui widgets
            imgui_sdl2_context.prepare_frame(
                imgui_context.io_mut(),
                &self.window,
                &event_pump.mouse_state(),
            );
            let ui = imgui_context.frame();

            self.draw_imgui(&ui); // 情報表示のみ
            image_manager = presenter.draw_imgui(&ui, image_manager); // event取得

            imgui_sdl2_context.prepare_render(&ui, &self.window);
            renderer.render(ui);

            self.window.gl_swap_window();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
        Ok(())
    }

    fn draw(&self, texture_id: u32) {
        let shader_id = self.screen_shader.get_shader_id();
        let (width, height) = self.window.size();
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::BLEND);
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            // gl::Disable(gl::CULL_FACE);

            gl::Viewport(0, 0, width as i32, height as i32);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(shader_id);

            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            draw!(self, gl::TRIANGLES);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    fn draw_imgui(&self, ui: &imgui::Ui) {
        imgui::Window::new(im_str!("Information"))
            .size([300.0, 450.0], imgui::Condition::FirstUseEver)
            .position([10.0, 10.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(im_str!("Viewer ver1.0"));
                ui.separator();
                ui.text(im_str!("FPS : {:.1}", ui.io().framerate));
                let display_size = ui.io().display_size;
                ui.text(format!(
                    "Display Size: ({:.1}, {:.1})",
                    display_size[0], display_size[1]
                ));
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse Positioin : ({:.1}, {:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));

                ui.separator();
            });
    }
}
