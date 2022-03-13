use std::cell::Cell;

use cgmath::Point3;
use imgui::im_str;

use crate::{
    model::Drawable,
    shader::{point_shader::PointShader, Shader},
};

use super::{build_pointlike_cloud, PointLike};

/// 画像上の点群の情報を保持するstruct.
/// `points`に保持される点は正規化座標系上の点である。
/// (画像の左下を(-1.0, -1.0)、右上を(1.0, 1.0)で中心を(0, 0)とする座標系)
pub struct Points {
    points: Vec<Point>,
    vao: u32,
    vbo: u32,
    vertex_num: u32,
    shader: Cell<Box<dyn Shader>>,
    draw_flag: bool,
    associated: Vec<Box<dyn Drawable>>,
}

impl Points {
    pub fn new() -> Box<Self> {
        Box::new(Points {
            points: Vec::new(),
            vao: 0,
            vbo: 0,
            vertex_num: 0,
            shader: Cell::new(Box::new(PointShader::new())),
            draw_flag: true,
            associated: Vec::new(),
        })
    }

    pub fn add_point(&mut self, x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) {
        self.points.push(Point::new(x, y, z, r, g, b));
    }

    /// 指定した座標に点が登録されているか判定する
    pub fn is_exist_point(&self, x: f32, y: f32) -> bool {
        for pt in &self.points {
            if pt.is_equal_to(x, y) {
                return true;
            }
        }
        false
    }

    pub fn set_point_size(&mut self, _pt_size: f32) {
        // self.shader.get_mut().set_point_size(pt_size);
    }

    pub fn get_point_size(&self) -> f32 {
        // self.shader.get_mut().get_point_size()
        1.0
    }
}

impl Drawable for Points {
    fn get_drawable_type(&self) -> super::DrawableType {
        super::DrawableType::Points
    }

    fn get_vertex_num(&self) -> u32 {
        self.vertex_num
    }

    fn get_draw_type(&self) -> gl::types::GLenum {
        gl::POINTS
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
            build_pointlike_cloud(&self.points, vec![gl::FLOAT, gl::FLOAT], vec![3, 3]);
        self.vao = vao;
        self.vbo = vbo;
        self.vertex_num = vertex_num;
    }

    fn draw_imgui(&mut self, ui: &imgui::Ui) {
        ui.separator();
        ui.text(im_str!("Point parameter"));
        let mut flag = !self.is_draw();
        if ui.checkbox(im_str!("Hide points"), &mut flag) {
            self.draw_flag = !flag;
        }
        self.get_mut_shader().draw_imgui(ui);
    }
}

/// 点情報を保持する
/// locには画像の中心を原点(0, 0)、右上を(1, 1)とした座標系での値を保持する。
pub struct Point {
    loc: Point3<f32>,
    color: (f32, f32, f32), // r, g, b value (range from 0.0 to 1.0).
}

impl Point {
    /// Retrun a point object.
    /// Arguments `x`, `y` and `z` are treated as point on the normalized coordinate system
    /// in which value range is from -1.0 to 1.0 with image center as (0, 0).
    /// Argument `r`, `g` and `b` are pixel values range from 0.0 to 1.0.
    pub fn new(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) -> Point {
        Point {
            loc: Point3::<f32> { x, y, z },
            color: (r, g, b),
        }
    }

    pub fn is_equal_to(&self, x: f32, y: f32) -> bool {
        (self.loc.x - x).abs() < 1e-5 && (self.loc.y - y).abs() < 1e-5
    }
}

impl PointLike for Point {
    fn to_vec(&self) -> Vec<f32> {
        vec![
            self.loc.x,
            self.loc.y,
            self.loc.z,
            self.color.0,
            self.color.1,
            self.color.2,
        ]
    }
}
