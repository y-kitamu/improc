use std::{
    cell::Cell,
    ffi::c_void,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use imgui::im_str;
use log::info;

use crate::{
    model::Model,
    view::{change_view_mode, View, VIEW_MODE_NAMES},
};

use super::Presenter;

pub struct ViewerPresenter {
    model: Cell<Box<dyn Model>>,
    view: Cell<Box<dyn View>>,
    imgui_context: Cell<imgui::Context>,
    imgui_sdl2_context: Cell<imgui_sdl2::ImguiSdl2>,
    imgui_renderer: imgui_opengl_renderer::Renderer,
    output_dir: PathBuf,
}

impl ViewerPresenter {
    pub fn new(model: Box<dyn Model>, view: Box<dyn View>) -> Self {
        let window = view.get_window();
        let video_subsystem = view.get_video_subsystem();

        let mut imgui_context = imgui::Context::create();
        imgui_context.set_ini_filename(None);

        let mut imgui_sdl2_context = imgui_sdl2::ImguiSdl2::new(&mut imgui_context, window);
        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui_context, |s| {
            video_subsystem.gl_get_proc_address(s) as _
        });
        ViewerPresenter {
            model: Cell::new(model),
            view: Cell::new(view),
            imgui_context: Cell::new(imgui_context),
            imgui_sdl2_context: Cell::new(imgui_sdl2_context),
            imgui_renderer: renderer,
            output_dir: Path::new(env!("CARGO_MANIFEST_DIR")).join("../outputs/screen_shots/"),
        }
    }

    fn save_screen(&self) -> Result<PathBuf> {
        let (width, height) = self.view.get_window().size();
        let data: Vec<u8> = vec![0; (width * height * 4) as usize];
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::ReadPixels(
                0,
                0,
                width as i32,
                height as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *mut c_void,
            );
        }

        if !self.output_dir.exists() {
            fs::create_dir_all(self.output_dir.as_path())?;
        }
        let mut idx = 0;
        while self.output_dir.join(format!("{:05}.png", idx)).exists() {
            idx += 1;
        }
        let output_path = self.output_dir.join(format!("{:05}.png", idx));

        image::imageops::flip_vertical(&image::RgbaImage::from_raw(width, height, data).unwrap())
            .save(output_path.as_path())?;
        Ok(output_path)
    }
}

impl Presenter for ViewerPresenter {
    fn get_model(&self) -> &Box<dyn Model> {
        &self.model
    }

    fn get_viewer(&self) -> &Box<dyn View> {
        &self.view
    }

    fn get_imgui_sdl2_context(&self) -> &imgui_sdl2::ImguiSdl2 {
        self.imgui_sdl2_context.get_mut()
    }

    fn get_mut_imgui_context(&self) -> &mut imgui::Context {
        self.imgui_context.get_mut()
    }

    fn prepare_imgui(&self) -> imgui::Ui {
        let window = self.get_viewer().get_window();
        let mouse_state = self.get_viewer().get_event_pump().mouse_state();
        self.imgui_sdl2_context.get_mut().prepare_frame(
            self.imgui_context.get_mut().io_mut(),
            window,
            &mouse_state,
        );
        self.imgui_context.get_mut().frame()
    }

    fn drwa_imgui(&mut self, ui: &imgui::Ui) {
        ui.main_menu_bar(|| {
            ui.menu(&im_str!("file"), true, || {
                if imgui::MenuItem::new(&im_str!("ScreenShot")).build(ui) {
                    println!("start save screenshot");
                    match self.save_screen() {
                        Ok(path) => {
                            info!(
                                "Success to save screen. Output to {}",
                                path.to_str().unwrap()
                            );
                        }
                        Err(_) => {
                            info!("Failed to save screen.");
                        }
                    }
                }
            })
        });

        ui.main_menu_bar(|| {
            ui.menu(&im_str!("modes"), true, || {
                for mode in VIEW_MODE_NAMES {
                    let selected = *mode == self.get_viewer().get_mode_name();
                    if imgui::MenuItem::new(&im_str!("{}", mode))
                        .selected(selected)
                        .build(ui)
                        && !selected
                    {
                        self.view = change_view_mode(self.view, mode);
                    }
                }
            })
        });

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

    fn render_imgui(&mut self, ui: imgui::Ui) {
        let window = self.get_viewer().get_window();
        self.imgui_sdl2_context
            .get_mut()
            .prepare_render(&ui, window);
        self.imgui_renderer.render(ui);
    }
}
