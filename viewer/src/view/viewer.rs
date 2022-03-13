use cgmath::One;
use imgui::im_str;
use sdl2::event::Event;

use crate::{
    model::{
        drawables::{screen::Screen, Drawable, DrawableType},
        Model,
    },
    shader::UniformVariable,
    utility::get_mouse_pos,
    Mat4,
};

use super::{initialize, View};

/// View of MVP architecture.
pub struct Viewer {
    sdl_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    _gl_context: sdl2::video::GLContext,
    event_pump: sdl2::EventPump,
    screen: Screen,
}

impl Viewer {
    pub const VIEWER_NAME: &'static str = "default";

    pub fn new(width: u32, height: u32) -> Box<Viewer> {
        let (sdl_context, video_subsystem, window, gl_context, event_pump) =
            initialize(width, height);
        Box::new(Viewer {
            sdl_context,
            video_subsystem,
            window,
            _gl_context: gl_context,
            event_pump,
            screen: Screen::new(width, height),
        })
    }

    pub fn change_from(from: Box<dyn View>) -> Box<Viewer> {
        let (sdl_context, video_subsystem, window, _gl_context, event_pump) = from.get_contexts();
        let (widht, height) = window.size();
        Box::new(Viewer {
            sdl_context,
            video_subsystem,
            window,
            _gl_context,
            event_pump,
            screen: Screen::new(widht, height),
        })
    }
}

impl View for Viewer {
    fn get_contexts(
        self,
    ) -> (
        sdl2::Sdl,
        sdl2::VideoSubsystem,
        sdl2::video::Window,
        sdl2::video::GLContext,
        sdl2::EventPump,
    ) {
        (
            self.sdl_context,
            self.video_subsystem,
            self.window,
            self._gl_context,
            self.event_pump,
        )
    }

    fn get_mode_name(&self) -> &str {
        Viewer::VIEWER_NAME
    }

    fn get_window(&self) -> &sdl2::video::Window {
        &self.window
    }

    fn get_video_subsystem(&self) -> &sdl2::VideoSubsystem {
        &self.video_subsystem
    }

    fn get_event_pump(&self) -> &sdl2::EventPump {
        &self.event_pump
    }

    fn set_image_list(&mut self, _image_num: usize) {}

    fn handle_event(&mut self, event: &sdl2::event::Event, model: &mut Box<dyn Model>) -> bool {
        let (fbo_width, fbo_height) = self.window.size();
        match event {
            Event::MouseWheel { y, direction, .. } => {
                let (mx, my) = get_mouse_pos();
                let cx = mx as f32 / fbo_width as f32 * 2.0 - 1.0;
                let cy = (fbo_height as f32 - my as f32) / fbo_height as f32 * 2.0 - 1.0;
                // let mut scale = 1.0f32 + *y as f32 / 10.0f32;
                // if *direction == MouseWheelDirection::Flipped {
                //     scale = 1.0f32 / scale;
                // }
                model.on_mouse_wheel(cx, cy, y, direction);
                true
            }
            Event::MouseButtonDown { x, y, .. } => {
                // 左上(0, 0), 右下(width, height)の座標系を
                // 中心(0, 0), 左上(-1.0, 1.0), 右下(1.0, -1.0)の座標系に変換する
                let fx = *x as f32 / fbo_width as f32 * 2.0f32 - 1.0f32;
                let fy = 1.0f32 - *y as f32 / fbo_height as f32 * 2.0f32;
                model.on_mouse_button_down(fx, fy);
                true
            }
            Event::MouseButtonUp { .. } => {
                model.on_mouse_button_up();
                true
            }
            Event::MouseMotion { xrel, yrel, .. } => {
                let dx = *xrel as f32 / fbo_width as f32 * 2.0f32;
                let dy = -*yrel as f32 / fbo_height as f32 * 2.0f32;
                model.on_mouse_motion_event(dx, dy);
                true
            }
            _ => false,
        }
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui, model: &mut Box<dyn Model>) {
        imgui::Window::new(im_str!("Mode parameters"))
            .size([300.0, 450.0], imgui::Condition::FirstUseEver)
            .position([400.0, 10.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(im_str!("Image parameter"));
                ui.separator();
                let new_img_idx: Option<usize> = None;
                for (idx, image) in model
                    .get_mut_drawables()
                    .iter()
                    .filter(|s| s.get_drawable_type() == DrawableType::Image)
                    .enumerate()
                {
                    let mut flag = image.is_draw();
                    if ui.radio_button(&im_str!("image {}", idx), &mut flag, true) {
                        image.set_is_draw(flag);
                    }
                }
                if let Some(idx) = new_img_idx {
                    for (i, image) in model
                        .get_mut_drawables()
                        .iter()
                        .filter(|s| s.get_drawable_type() == DrawableType::Image)
                        .enumerate()
                    {
                        if i != idx {
                            image.set_is_draw(false);
                        }
                    }
                }
                ui.separator();
            });
    }

    /// 描画先をframe bufferに設定する
    fn prepare_framebuffer(&mut self) {
        let (width, height) = self.window.size();
        self.screen.prepare(width, height);
    }

    /// frame bufferに画像を描画し、frame bufferを内容を画面に反映する
    fn draw(&self, model: &mut Box<dyn Model>) {
        let view_mat = UniformVariable::new("dummy", Mat4::one());
        let proj_mat = UniformVariable::new("dummy", Mat4::one());
        model.draw();
        self.screen.draw(&view_mat, &proj_mat);
    }
}
