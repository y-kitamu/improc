use std::{ffi::c_void, mem};

use gl::types::{GLfloat, GLsizei, GLsizeiptr};

use crate::{
    define_gl_primitive,
    model::register_primitive,
    shader::{image_shader::ImageShader, line_shader::LineShader},
};

use super::{Drawable, GLPrimitive};

const DEFAULT_LINE_SHADER_KEY: &str = "line";

pub struct Arrows {
    pub vao: Option<u32>,
    pub vbo: Option<u32>,
    pub vertex_num: i32,
    pub arrows: Vec<Arrow>,
    pub shader: LineShader,
}

define_gl_primitive!(Arrows);

impl Arrows {
    pub fn new() -> Self {
        Arrows {
            vao: None,
            vbo: None,
            vertex_num: 0,
            arrows: Vec::new(),
            shader: LineShader::new(DEFAULT_LINE_SHADER_KEY),
        }
    }

    /// add arrow to (x, y). direction (radian) = `dir`, length = `length`
    pub fn add_arrow(&mut self, x: f32, y: f32, dir: f32, length: f32) {
        self.arrows.push(Arrow::new(x, y, dir, length));
    }

    pub fn build(&mut self) {
        let buf_array: Vec<f32> = self
            .arrows
            .iter()
            .map(|arrow| arrow.to_vec())
            .flatten()
            .collect();
        let n_vertex_per_point = 3;
        let attribute_types = vec![gl::FLOAT];
        let attribute_sizes = vec![3];
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

    pub fn draw(&self, image_shader: &ImageShader) {}
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
            self.x, self.y, 1.0, tx, ty, 1.0, // center line
            tx, ty, 1.0, lx, ly, 1.0, // left wing
            tx, ty, 1.0, rx, ry, 1.0, // right wing
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::{shader::UniformVariable, Vector3};

    use super::*;

    #[test]
    fn test_arrows() {
        let mut arrs = Arrows {
            vao: Some(0),
            vbo: Some(1),
            vertex_num: 10,
            arrows: Vec::new(),
            shader: LineShader {
                id: 1,
                color: UniformVariable::new("uColor", Vector3::new(1.0, 0.0, 0.0)),
            },
        };

        arrs.add_arrow(1.0, 0.1, 0.0, 5.0);
        assert_eq!(arrs.arrows.len(), 1);
    }

    #[test]
    fn test_arrow() {
        let arr = Arrow::new(1.0, 0.5, std::f32::consts::FRAC_PI_2, 1.0);
        let vec = arr.to_vec();
        assert!((vec[0] - 1.0).abs() < 1e-5);
        assert!((vec[1] - 0.5).abs() < 1e-5);
        assert!((vec[2] - 1.0).abs() < 1e-5);
        assert!((vec[3] - 1.0).abs() < 1e-5);
        assert!((vec[4] - 1.5).abs() < 1e-5);
        assert!((vec[5] - 1.0).abs() < 1e-5);
        assert!((vec[9] - 0.9).abs() < 1e-5, "lhs x = {}", vec[9]);
        assert!((vec[10] - vec[16]).abs() < 1e-5);
        assert!(vec[10] < vec[4]);
        assert!((vec[15] - 1.1).abs() < 1e-5);
    }
}
