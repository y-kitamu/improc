use std::cell::Cell;

use imgui::im_str;

use crate::{
    model::Drawable,
    shader::{arrow_shader::ArrowShader, Shader},
};

use super::{build_pointlike_cloud, PointLike};

pub struct Arrows {
    arrows: Vec<Arrow>,
    vao: u32,
    vbo: u32,
    vertex_num: u32,
    shader: Cell<Box<dyn Shader>>,
    draw_flag: bool,
    associated: Vec<Box<dyn Drawable>>,
}

impl Arrows {
    pub fn new() -> Box<Self> {
        Box::new(Arrows {
            arrows: Vec::new(),
            vao: 0,
            vbo: 0,
            vertex_num: 0,
            shader: Cell::new(Box::new(ArrowShader::new())),
            draw_flag: false,
            associated: Vec::new(),
        })
    }

    /// add arrow to (x, y). direction (radian) = `dir`, length = `length`
    pub fn add_arrow(&mut self, x: f32, y: f32, dir: f32, length: f32) {
        self.arrows.push(Arrow::new(x, y, dir, length));
    }

    pub fn set_color(&mut self, _r: f32, _g: f32, _b: f32, _a: f32) {}
}

impl Drawable for Arrows {
    fn get_drawable_type(&self) -> super::DrawableType {
        super::DrawableType::Arrows
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
            build_pointlike_cloud(&self.arrows, vec![gl::FLOAT, gl::FLOAT], vec![3, 2]);
        self.vao = vao;
        self.vbo = vbo;
        self.vertex_num = vertex_num;
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui) {
        ui.separator();
        ui.text(im_str!("Arrows parameter"));
        let mut flag = !self.is_draw();
        if ui.checkbox(im_str!("Hide arrows"), &mut flag) {
            self.draw_flag = !flag;
        }
        self.get_mut_shader().draw_imgui(ui);
    }
}

/// x, y, length はnormalized coordinate (-1.0 ~ 1.0), directionはradian単位
pub struct Arrow {
    x: f32,
    y: f32,
    direction: f32,
    length: f32,
}

impl Arrow {
    fn new(x: f32, y: f32, direction: f32, length: f32) -> Self {
        Arrow {
            x,
            y,
            direction,
            length,
        }
    }
}

impl PointLike for Arrow {
    fn to_vec(&self) -> Vec<f32> {
        let tx = self.x + self.length * self.direction.cos();
        let ty = self.y + self.length * self.direction.sin();
        let lrad = std::f32::consts::PI + self.direction - std::f32::consts::FRAC_PI_6;
        let rrad = std::f32::consts::PI + self.direction + std::f32::consts::FRAC_PI_6;
        let lx = tx + self.length * 0.2 * lrad.cos();
        let ly = ty + self.length * 0.2 * lrad.sin();
        let rx = tx + self.length * 0.2 * rrad.cos();
        let ry = ty + self.length * 0.2 * lrad.sin();
        vec![
            self.x, self.y, 1.0, self.x, self.y, tx, ty, 1.0, self.x, self.y, // center line
            tx, ty, 1.0, self.x, self.y, lx, ly, 1.0, self.x, self.y, // left wing
            tx, ty, 1.0, self.x, self.y, rx, ry, 1.0, self.x, self.y, // right wing
        ]
    }
}
