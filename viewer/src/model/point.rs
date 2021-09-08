use std::{ffi::c_void, mem};

use cgmath::Point3;
use gl::types::{GLfloat, GLsizei, GLsizeiptr};

use crate::{
    define_gl_primitive, draw,
    model::register_primitive,
    shader::{image_shader::ImageShader, point_shader::PointShader},
};

use super::GLPrimitive;

const DEFAULT_POINTS_SHADER_KEY: &str = "points";

/// 画像上の点群の情報を保持するstruct.
/// `points`に保持される点は正規化座標系上の点である。
/// (画像の左下を(-1.0, -1.0)、右上を(1.0, 1.0)で中心を(0, 0)とする座標系)
pub struct Points {
    points: Vec<Point>,
    vao: Option<u32>,
    vbo: Option<u32>,
    vertex_num: i32,
    shader: PointShader,
}

define_gl_primitive!(Points);

impl Points {
    pub fn new() -> Self {
        Points {
            points: Vec::new(),
            vao: None,
            vbo: None,
            vertex_num: 0,
            shader: PointShader::new(DEFAULT_POINTS_SHADER_KEY),
        }
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

    pub fn build(&mut self) {
        if self.points.len() > 0 && self.vao.is_none() {
            let n_vertex_per_point = self.points[0].to_vec().len();
            let attribute_types = vec![gl::FLOAT, gl::FLOAT];
            let attribute_sizes = vec![3, 3];
            let buf_array = self
                .points
                .iter()
                .map(|p| p.to_vec())
                .flatten()
                .collect::<Vec<f32>>();
            let (vao, vbo) = register_primitive(
                (buf_array.len() as usize * mem::size_of::<GLfloat>()) as GLsizeiptr,
                buf_array.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
                attribute_types,
                attribute_sizes,
                (n_vertex_per_point * mem::size_of::<GLfloat>()) as GLsizei,
            );
            self.vao = Some(vao);
            self.vbo = Some(vbo);
            self.vertex_num = (buf_array.len() / n_vertex_per_point) as i32;
        }
    }

    pub fn draw(&self, image_shader: &ImageShader) {
        self.shader.set_uniform_variables(image_shader);
        draw!(self, gl::POINTS);
        unsafe {
            gl::UseProgram(0);
        }
    }
}

struct Color {
    r: f32,
    g: f32,
    b: f32,
}

/// 点情報を保持する
/// locには画像の中心を原点(0, 0)、右上を(1, 1)とした座標系での値を保持する。
struct Point {
    loc: Point3<f32>,
    color: Color,
}

impl Point {
    /// Retrun a point object.
    /// Arguments `x`, `y` and `z` are treated as point on the normalized coordinate system
    /// in which value range is from -1.0 to 1.0 with image center as (0, 0).
    /// Argument `r`, `g` and `b` are pixel values range from 0.0 to 1.0.
    pub fn new(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) -> Point {
        Point {
            loc: Point3::<f32> { x, y, z },
            color: Color { r, g, b },
        }
    }

    pub fn is_equal_to(&self, x: f32, y: f32) -> bool {
        (self.loc.x - x).abs() < 1e-5 && (self.loc.y - y).abs() < 1e-5
    }

    pub fn to_vec(&self) -> Vec<f32> {
        vec![
            self.loc.x,
            self.loc.y,
            self.loc.z,
            self.color.r,
            self.color.g,
            self.color.b,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_point() {
        let pt = Point::new(1.0, 0.5, -1.0, 0.0, 1.0, 0.8);
        assert!((pt.loc.x - 1.0).abs() < 1e-5);
        assert!((pt.loc.y - 0.5).abs() < 1e-5);
        assert!((pt.loc.z + 1.0).abs() < 1e-5);
        assert!((pt.color.r - 0.0).abs() < 1e-5);
        assert!((pt.color.g - 1.0).abs() < 1e-5);
        assert!((pt.color.b - 0.8).abs() < 1e-5);

        assert!(pt.is_equal_to(1.0, 0.5));
        assert!(!pt.is_equal_to(1.0, 0.55));
    }
}
