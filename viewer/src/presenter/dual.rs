use std::collections::HashMap;

use imgui::im_str;
use sdl2::{event::Event, mouse::MouseWheelDirection, sys::SDL_GetMouseState};

use crate::{
    model::image_manager::ImageManager,
    shader::{image_shader::ImageShader, line_shader::LineShader, point_shader::PointShader},
    utility::{get_mouse_pos, scale_matrix},
    vertex::Vertex,
};

use super::PresenterMode;

const SHADER_LIST: [&str; 1] = ["default"];
const POINTS_SHADER_LIST: [&str; 1] = ["points"];

const DEFAULT_SHADER_KEY: &str = "default";
const DEFAULT_POINTS_SHADER_KEY: &str = "points";
const DEFAULT_LINE_SHADER_KEY: &str = "line";

pub struct DualImagePresenter {
    shader_map_left: HashMap<String, ImageShader>,
    shader_map_right: HashMap<String, ImageShader>,
    points_shader_map: HashMap<String, PointShader>,
    current_shader_key: String,
    current_image_keys: (String, String),
    current_points_shader_key: String,
    point_relation_shader: LineShader,
}

impl DualImagePresenter {
    const MODE_NAME: &'static str = "dual";

    pub fn new() -> Self {
        let shader_map_left = load_shaders!(SHADER_LIST, ImageShader);
        let shader_map_right = load_shaders!(SHADER_LIST, ImageShader);
        let points_shader_map = load_shaders!(POINTS_SHADER_LIST, PointShader);
        let current_shader_key = DEFAULT_SHADER_KEY.to_string();
        let current_image_keys = ("".to_string(), "".to_string());
        let current_points_shader_key = DEFAULT_POINTS_SHADER_KEY.to_string();
        let point_relation_shader = LineShader::new(DEFAULT_LINE_SHADER_KEY);
        DualImagePresenter {
            shader_map_left,
            shader_map_right,
            points_shader_map,
            current_shader_key,
            current_image_keys,
            current_points_shader_key,
            point_relation_shader,
        }
    }

    // TODO: refactor. default.rsの`draw`とほぼおなじコード
    fn draw_half(
        &mut self,
        image_key: &str,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
        image_manager: &ImageManager,
        fbo_vertex: &Vertex,
    ) {
        let image_texture_id = image_manager.get_texture_id(image_key);
        let (image_width, image_height) = image_manager.get_texture_image_size(image_key);

        let shader = if left == 0 {
            self.shader_map_left
                .get_mut(&self.current_shader_key)
                .unwrap()
        } else {
            self.shader_map_right
                .get_mut(&self.current_shader_key)
                .unwrap()
        };
        shader.adjust_aspect_ratio(image_width, image_height, width, height);

        let pts_vertex = image_manager.get_points_vertex(image_key);
        let pts_shader = self
            .points_shader_map
            .get(&self.current_points_shader_key)
            .unwrap();

        unsafe {
            gl::Viewport(left as i32, top as i32, width as i32, height as i32);

            shader.set_uniform_variables();
            gl::BindTexture(gl::TEXTURE_2D, image_texture_id);
            fbo_vertex.draw();
            gl::BindTexture(gl::TEXTURE_2D, 0);

            if let Some(pts_vtx) = pts_vertex {
                pts_shader.set_uniform_variables(&shader);
                pts_vtx.draw_points();
            }
        }
    }

    fn get_current_shader(&mut self, fbo_width: u32) -> &mut ImageShader {
        let (x, _y) = get_mouse_pos();
        let current_shader = if (x as u32) < fbo_width / 2 {
            self.shader_map_left
                .get_mut(&self.current_shader_key)
                .unwrap()
        } else {
            self.shader_map_right
                .get_mut(&self.current_shader_key)
                .unwrap()
        };
        current_shader
    }
}

impl PresenterMode for DualImagePresenter {
    fn get_mode_name(&self) -> &str {
        Self::MODE_NAME
    }

    fn process_event(&mut self, event: &Event, fbo_width: u32, fbo_height: u32) -> bool {
        // let current_shader = self.get_current_shader();
        let processed = match event {
            Event::MouseWheel {
                timestamp,
                window_id,
                which,
                x,
                y,
                direction,
            } => {
                let (mx, my) = get_mouse_pos();
                let half = fbo_width as f32 / 2.0;
                let cy = (fbo_height as f32 - my as f32) / fbo_height as f32 * 2.0 - 1.0;
                let cx = if (mx as u32) < fbo_width / 2 {
                    (mx as f32) / half * 2.0 - 1.0
                } else {
                    (mx as f32 - half) / half * 2.0 - 1.0
                };
                let mut scale = 1.0f32 + *y as f32 / 10.0f32;
                if *direction == MouseWheelDirection::Flipped {
                    scale = 1.0f32 / scale;
                }
                let current_shader = self.get_current_shader(fbo_width);
                current_shader.model_mat.value =
                    scale_matrix(&current_shader.model_mat.value, cx, cy, scale);
                true
            }
            Event::MouseButtonDown {
                timestamp,
                window_id,
                which,
                mouse_btn,
                clicks,
                x,
                y,
            } => {
                // 左上(0, 0), 右下(width, height)の座標系を
                // 中心(0, 0), 左上(-1.0, 1.0), 右下(1.0, -1.0)の座標系に変換する
                let fx = if (*x as u32) < fbo_width / 2 {
                    *x as f32 / fbo_width as f32 * 4.0f32 - 1.0f32
                } else {
                    (*x as f32 / fbo_width as f32 - 0.5) * 4.0f32 - 1.0f32
                };
                let fy = 1.0f32 - *y as f32 / fbo_height as f32 * 2.0f32;
                let current_shader = self.get_current_shader(fbo_width);
                current_shader
                    .on_mouse_button_down(timestamp, window_id, which, mouse_btn, clicks, fx, fy);
                true
            }
            Event::MouseButtonUp {
                timestamp,
                window_id,
                which,
                mouse_btn,
                clicks,
                x,
                y,
            } => {
                let current_shader = self.get_current_shader(fbo_width);
                current_shader
                    .on_mouse_button_up(timestamp, window_id, which, mouse_btn, clicks, x, y);
                true
            }
            Event::MouseMotion {
                timestamp,
                window_id,
                which,
                mousestate,
                x,
                y,
                xrel,
                yrel,
            } => {
                let current_shader = self.get_current_shader(fbo_width);
                let dx = *xrel as f32 / fbo_width as f32 * 4.0f32;
                let dy = -*yrel as f32 / fbo_height as f32 * 2.0f32;
                current_shader
                    .on_mouse_motion_event(timestamp, window_id, which, mousestate, x, y, dx, dy);
                true
            }
            _ => false,
        };
        processed
    }

    fn draw(&mut self, width: u32, height: u32, image_manager: &ImageManager, fbo_vertex: &Vertex) {
        if self.current_image_keys.0.len() == 0 || self.current_image_keys.1.len() == 0 {
            return;
        }

        let cur_img_key0 = self.current_image_keys.0.clone();
        self.draw_half(
            &cur_img_key0,
            0,
            0,
            width / 2,
            height,
            image_manager,
            fbo_vertex,
        );
        let cur_img_key1 = self.current_image_keys.1.clone();
        self.draw_half(
            &cur_img_key1,
            width / 2,
            0,
            width / 2,
            height,
            image_manager,
            fbo_vertex,
        );

        // draw line
        let lhs_img_shader = self.shader_map_left.get(&self.current_shader_key).unwrap();
        let rhs_img_shader = self.shader_map_right.get(&self.current_shader_key).unwrap();
        let line_vertex = image_manager.get_point_relation(&cur_img_key0, &cur_img_key1);
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            self.point_relation_shader
                .set_uniform_variables(lhs_img_shader, rhs_img_shader);
            if let Some(lvtx) = line_vertex {
                lvtx.draw_lines();
            }
            gl::UseProgram(0);
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
                imgui::Slider::new(im_str!("Point size"))
                    .range(1.0..=100.0)
                    .build(
                        &ui,
                        &mut self
                            .points_shader_map
                            .get_mut(&self.current_points_shader_key)
                            .unwrap()
                            .point_size
                            .value,
                    );
                ui.separator();

                ui.text(im_str!("Line parameter"));
                imgui::Slider::new(im_str!("Color (R)"))
                    .range(0.0..=1.0)
                    .build(&ui, &mut self.point_relation_shader.color.value.x);
                imgui::Slider::new(im_str!("Color (G)"))
                    .range(0.0..=1.0)
                    .build(&ui, &mut self.point_relation_shader.color.value.y);
                imgui::Slider::new(im_str!("Color (B)"))
                    .range(0.0..=1.0)
                    .build(&ui, &mut self.point_relation_shader.color.value.z);
            });
    }
}
