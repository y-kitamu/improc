use std::collections::HashMap;

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
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui, image_manager: &ImageManager) {}
}
