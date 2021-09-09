use std::{ffi::c_void, mem};

use gl::types::{GLfloat, GLsizei, GLsizeiptr};

use crate::{define_drawable, define_gl_primitive, model::register_primitive};

use super::{Drawable, GLPrimitive};

pub struct PointRelations {
    pub vao: Option<u32>,
    pub vbo: Option<u32>,
    pub vertex_num: i32,
    pub lines: Vec<Line>,
}

define_gl_primitive!(PointRelations);
define_drawable!(PointRelations, gl::LINES);

impl PointRelations {
    pub fn new() -> Self {
        PointRelations {
            vao: None,
            vbo: None,
            vertex_num: 0,
            lines: Vec::new(),
        }
    }

    pub fn add_relation(&mut self, x: f32, y: f32, other_x: f32, other_y: f32) {
        self.lines.push(Line {
            x,
            y,
            other_x,
            other_y,
        });
    }

    pub fn build(&mut self) {
        let attrib_types = vec![gl::FLOAT, gl::FLOAT];
        let attrib_sizes = vec![3, 1];
        let block_size = attrib_sizes.iter().fold(0, |a, b| a + b) as usize;
        let buf_array: Vec<f32> = self
            .lines
            .iter()
            .map(|line| line.to_vec())
            .flatten()
            .collect();
        let (vao, vbo) = register_primitive(
            (buf_array.len() as usize * mem::size_of::<GLfloat>()) as GLsizeiptr,
            buf_array.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
            attrib_types,
            attrib_sizes,
            (block_size * mem::size_of::<GLfloat>()) as GLsizei,
        );
        self.vao = Some(vao);
        self.vbo = Some(vbo);
        self.vertex_num = (buf_array.len() / block_size) as i32;
    }
}

pub struct Line {
    x: f32,
    y: f32,
    other_x: f32,
    other_y: f32,
}

impl Line {
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
