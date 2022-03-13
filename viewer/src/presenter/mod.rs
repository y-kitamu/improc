pub mod presenter;

use anyhow::Result;
use sdl2::{event::Event, keyboard::Keycode};

use crate::{model::Model, view::View};

pub trait Presenter {
    fn get_model(&self) -> &Box<dyn Model>;
    fn get_viewer(&self) -> &Box<dyn View>;
    fn get_imgui_sdl2_context(&self) -> &imgui_sdl2::ImguiSdl2;
    fn get_mut_imgui_context(&self) -> &mut imgui::Context;

    fn render(&mut self) -> Result<()> {
        let mut model = self.get_model();
        let mut viewer = self.get_viewer();

        model.build();

        'running: loop {
            for event in viewer.get_event_pump().poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                };
                let imgui_sdl2_context = self.get_imgui_sdl2_context();
                imgui_sdl2_context.handle_event(self.get_mut_imgui_context(), &event);
                if imgui_sdl2_context.ignore_event(&event) {
                    break 'running;
                }
                if viewer.handle_event(&event, &mut model) {
                    continue;
                }
            }
            // draw image to fbo
            viewer.prepare_framebuffer(); // prepare framebuffer to be drawn
            viewer.draw(&mut model); // draw framebuffer to display

            let ui = self.prepare_imgui();
            viewer.draw_imgui(&ui, &mut model);
            for obj in model.get_mut_drawables() {
                if obj.is_draw() {
                    obj.draw_imgui(&ui)
                }
            }
            self.render_imgui(ui);
            viewer.on_step_end();
        }
        Ok(())
    }

    fn prepare_imgui(&self) -> imgui::Ui;

    fn drwa_imgui(&mut self, ui: &imgui::Ui);

    fn render_imgui(&mut self, ui: imgui::Ui);
}
