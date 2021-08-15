use std::collections::HashMap;

use imgui::im_str;
use sdl2::event::Event;

use crate::{
    image_manager::ImageManager,
    shader::{self, Shader},
    vertex::Vertex,
};

use super::{Presenter, PresenterMode};

const SHADER_LIST: [&str; 1] = ["default"];
const POINTS_SHADER_LIST: [&str; 1] = ["points"];

const DEFAULT_SHADER_KEY: &str = "default";
const DEFAULT_POINTS_SHADER_KEY: &str = "points";

pub struct DualImagePresenter {
    shader_map: HashMap<String, Shader>,
    points_shader_map: HashMap<String, Shader>,
    current_shader_key: String,
    current_image_keys: (String, String),
    current_points_shader_key: String,
}

impl DualImagePresenter {
    const MODE_NAME: &'static str = "dual";

    pub fn new() -> Self {
        let shader_map = shader::load_shaders(&SHADER_LIST.to_vec());
        let points_shader_map = shader::load_shaders(&POINTS_SHADER_LIST.to_vec());
        let current_shader_key = DEFAULT_SHADER_KEY.to_string();
        let current_image_keys = ("".to_string(), "".to_string());
        let current_points_shader_key = DEFAULT_POINTS_SHADER_KEY.to_string();
        DualImagePresenter {
            shader_map,
            points_shader_map,
            current_shader_key,
            current_image_keys,
            current_points_shader_key,
        }
    }

    fn draw_half(
        &mut self,
        image_key: &str,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
        image_manager: &ImageManager,
        fbo_id: u32,
        fbo_vertex: &Vertex,
    ) {
        let image_texture_id = image_manager.get_texture_id(image_key);
        let (image_width, image_height) = image_manager.get_texture_image_size(image_key);

        let shader = self.shader_map.get_mut(&self.current_shader_key).unwrap();
        shader.adjust_aspect_ratio(image_width, image_height, width, height);
        let shader_id = shader.get_shader_id();

        let points_vertex = image_manager.get_points_vertex(image_key);
        let points_shader_id = self
            .points_shader_map
            .get(&self.current_points_shader_key)
            .unwrap()
            .get_shader_id();

        unsafe {
            gl::Viewport(left as i32, top as i32, width as i32, height as i32);

            gl::UseProgram(shader_id);
            shader.set_uniform_variables(shader_id, false);

            gl::BindTexture(gl::TEXTURE_2D, image_texture_id);
            fbo_vertex.draw();
            gl::BindTexture(gl::TEXTURE_2D, 0);

            if let Some(pts_vtx) = points_vertex {
                gl::UseProgram(points_shader_id);
                shader.set_uniform_variables(points_shader_id, true);
                pts_vtx.draw_points();
            }
        }
    }
}

impl PresenterMode for DualImagePresenter {
    fn get_mode_name(&self) -> &str {
        Self::MODE_NAME
    }

    fn process_event(&mut self, event: &Event, fbo_width: u32, fbo_height: u32) -> bool {
        false
    }

    fn draw(
        &mut self,
        width: u32,
        height: u32,
        image_manager: &ImageManager,
        fbo_id: u32,
        fbo_vertex: &Vertex,
    ) {
        if self.current_image_keys.0.len() == 0 || self.current_image_keys.1.len() == 0 {
            return;
        }

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo_id);
            gl::Enable(gl::PROGRAM_POINT_SIZE);

            gl::Viewport(0, 0, width as i32, height as i32);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let cur_img_key0 = self.current_image_keys.0.clone();
        self.draw_half(
            &cur_img_key0,
            0,
            0,
            width / 2,
            height,
            image_manager,
            fbo_id,
            fbo_vertex,
        );
        let cur_img_key1 = self.current_image_keys.0.clone();
        self.draw_half(
            &cur_img_key1,
            width / 2,
            0,
            width / 2,
            height,
            image_manager,
            fbo_id,
            fbo_vertex,
        );

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui, image_manager: &ImageManager) {
        imgui::Window::new(im_str!("Parameters"))
            .size([200.0, 250.0], imgui::Condition::FirstUseEver)
            .position([700.0, 10.0], imgui::Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("Image parameter"));
                ui.separator();

                for key in image_manager.get_image_keys() {
                    let mut flag = self.current_image_keys.0 == *key;
                    if ui.radio_button(&im_str!("{}0", key), &mut flag, true) {
                        self.current_image_keys.0 = key.clone();
                    }
                    ui.same_line(100.0);
                    let mut flag = self.current_image_keys.1 == *key;
                    if ui.radio_button(&im_str!("{}1", key), &mut flag, true) {
                        self.current_image_keys.1 = key.clone();
                    }
                }

                ui.separator();
                ui.text(im_str!("Point parameter"));
                let shader = self.shader_map.get_mut(&self.current_shader_key).unwrap();
                imgui::Slider::new(im_str!("Point size"))
                    .range(1.0..=100.0)
                    .build(&ui, &mut shader.point_size.value);
                ui.separator();
            });
    }
}
