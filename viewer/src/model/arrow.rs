use crate::{define_drawable, define_gl_primitive};

use super::{Drawable, GLPrimitive};

pub struct Arrows {
    vao: Option<u32>,
    vbo: Option<u32>,
    vertex_num: i32,
    arrows: Vec<Arrow>,
}

impl Arrows {
    pub fn new() -> Self {
        Arrows {
            vao: None,
            vbo: None,
            vertex_num: 0,
            arrows: Vec::new(),
        }
    }

    pub fn build(&mut self) {
        let buf_arr: Vec<f32> = self
            .arrows
            .iter()
            .map(|arrow| arrow.to_vec())
            .flatten()
            .collect();
    }
}

define_gl_primitive!(Arrows);
define_drawable!(Arrows, gl::LINES);

struct Arrow {
    x: f32,
    y: f32,
    direction: f32,
    length: f32,
}

impl Arrow {
    fn to_vec(&self) -> Vec<f32> {
        vec![]
    }
}
