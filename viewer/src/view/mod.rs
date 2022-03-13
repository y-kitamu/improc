use std::time::Duration;

use log::info;
use sdl2::sys::SDL_SetWindowResizable;

use crate::model::Model;

use self::viewer::Viewer;

pub mod viewer;

pub const VIEW_MODE_NAMES: &[&str] = &[Viewer::VIEWER_NAME];

pub fn change_view_mode(from: Box<dyn View>, to_name: &str) -> Box<dyn View> {
    match to_name {
        Viewer::VIEWER_NAME => Viewer::change_from(from),
        _ => from,
    }
}

pub trait View {
    fn get_contexts(
        self,
    ) -> (
        sdl2::Sdl,
        sdl2::VideoSubsystem,
        sdl2::video::Window,
        sdl2::video::GLContext,
        sdl2::EventPump,
    );
    fn get_mode_name(&self) -> &str;
    fn get_window(&self) -> &sdl2::video::Window;
    fn get_video_subsystem(&self) -> &sdl2::VideoSubsystem;
    fn get_event_pump(&self) -> &sdl2::EventPump;
    ///
    fn set_image_list(&mut self, image_num: usize);
    /// handle event and return false if the event should be passed to another handler, else true.
    fn handle_event(&mut self, event: &sdl2::event::Event, model: &mut Box<dyn Model>) -> bool;
    fn draw_imgui(&mut self, ui: &imgui::Ui, model: &mut Box<dyn Model>);
    fn prepare_framebuffer(&mut self);
    fn draw(&self, model: &mut Box<dyn Model>);
    fn on_step_end(&self) {
        self.get_window().gl_swap_window();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn initialize(
    width: u32,
    height: u32,
) -> (
    sdl2::Sdl,
    sdl2::VideoSubsystem,
    sdl2::video::Window,
    sdl2::video::GLContext,
    sdl2::EventPump,
) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    {
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 1);
        let (major, minor) = gl_attr.context_version();
        println!("OK : init OpenGL: version = {}.{}", major, minor);
    }
    let window = video_subsystem
        .window("SDL", width, height)
        .opengl()
        .position_centered()
        .build()
        .unwrap();
    unsafe {
        SDL_SetWindowResizable(window.raw(), sdl2::sys::SDL_bool::SDL_TRUE);
    }
    let gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);
    info!("OK : Initialize SDL and GL.");
    (
        sdl_context,
        video_subsystem,
        window,
        gl_context,
        sdl_context.event_pump().unwrap(),
    )
}
