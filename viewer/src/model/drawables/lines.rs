use std::cell::Cell;

use imgui::im_str;

use crate::{
    model::Drawable,
    shader::{line_shader::LineShader, Shader},
};

use super::{build_pointlike_cloud, PointLike};

pub struct Lines {
    vao: u32,
    vbo: u32,
    vertex_num: u32,
    shader: Cell<Box<dyn Shader>>,
    lines: Vec<Line>,
    draw_flag: bool,
    associated: Vec<Box<dyn Drawable>>,
}

impl Lines {
    pub fn new() -> Box<Self> {
        Box::new(Lines {
            vao: 0,
            vbo: 0,
            vertex_num: 0,
            shader: Cell::new(Box::new(LineShader::new())),
            lines: Vec::new(),
            draw_flag: false,
            associated: Vec::new(),
        })
    }

    pub fn add_line(&mut self, x: f32, y: f32, other_x: f32, other_y: f32) {
        self.lines.push(Line::new(x, y, other_x, other_y));
    }
}

impl Drawable for Lines {
    fn get_drawable_type(&self) -> super::DrawableType {
        super::DrawableType::Line
    }

    fn get_vertex_num(&self) -> u32 {
        self.vertex_num
    }

    fn get_draw_type(&self) -> gl::types::GLenum {
        gl::LINES
    }

    fn get_model_mat(&mut self) -> crate::Mat4 {
        self.shader.get_mut().get_model_mat().value.clone()
    }

    fn get_mut_shader(&mut self) -> &mut Box<dyn crate::shader::Shader> {
        self.shader.get_mut()
    }

    fn get_associated_drawables(&mut self) -> &Vec<Box<dyn Drawable>> {
        &self.associated
    }

    fn get_mut_associated_drawables(&mut self) -> &mut Vec<Box<dyn Drawable>> {
        &mut self.associated
    }

    fn is_draw(&self) -> bool {
        self.draw_flag
    }

    fn set_is_draw(&mut self, flag: bool) {
        self.draw_flag = flag;
    }

    fn get_vao(&self) -> u32 {
        self.vao
    }

    fn get_texture_id(&self) -> u32 {
        0
    }

    fn build(&mut self) {
        let (vao, vbo, vertex_num) =
            build_pointlike_cloud(&self.lines, vec![gl::FLOAT, gl::FLOAT], vec![3, 1]);
        self.vao = vao;
        self.vbo = vbo;
        self.vertex_num = vertex_num;
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui) {
        ui.separator();
        ui.text(im_str!("Line parameter"));

        let mut flag = !self.is_draw();
        if ui.checkbox(im_str!("Hide lines"), &mut flag) {
            self.draw_flag = !flag;
        }
    }
}

pub struct Line {
    x: f32,
    y: f32,
    other_x: f32,
    other_y: f32,
}

impl Line {
    pub fn new(x: f32, y: f32, other_x: f32, other_y: f32) -> Self {
        Line {
            x,
            y,
            other_x,
            other_y,
        }
    }
}

impl PointLike for Line {
    fn to_vec(&self) -> Vec<f32> {
        vec![
            self.x,
            self.y,
            1.0,
            0.0,
            self.other_x,
            self.other_y,
            1.0,
            1.0,
        ]
    }
}
