mod arrow;
mod image;
pub mod image_manager;
mod point;
mod point_relation;

use std::mem;
use std::os::raw::c_void;

use gl::types::{GLenum, GLfloat, GLint, GLsizei, GLsizeiptr};

pub trait GLPrimitive {
    fn vao(&self) -> u32;

    fn vbo(&self) -> u32;

    fn vertex_num(&self) -> i32;
}

#[macro_export]
macro_rules! define_gl_primitive {
    ($t:ty) => {
        impl GLPrimitive for $t {
            fn vao(&self) -> u32 {
                match self.vao {
                    Some(vao) => vao,
                    None => 0,
                }
            }

            fn vbo(&self) -> u32 {
                match self.vbo {
                    Some(vao) => vao,
                    None => 0,
                }
            }

            fn vertex_num(&self) -> i32 {
                self.vertex_num
            }
        }
    };
}

trait Drawable {
    fn draw(&self);
}

#[macro_export]
macro_rules! draw {
    ($e:expr, $p:path) => {
        unsafe {
            gl::BindVertexArray($e.vao());
            gl::DrawArrays($p, 0, $e.vertex_num());
            gl::BindVertexArray(0);
        }
    };
}

#[macro_export]
macro_rules! define_drawable {
    ($t:ty, $p:path) => {
        impl Drawable for $t {
            fn draw(&self) {
                unsafe {
                    gl::BindVertexArray(self.vao());
                    gl::DrawArrays($p, 0, self.vertex_num());
                    gl::BindVertexArray(0);
                }
            }
        }
    };
}

/// OpenGLのprimitiveを作成、vao, vboを返す
fn register_primitive(
    size: GLsizeiptr,
    data: *const c_void,
    usage: GLenum,
    attribute_type_vec: std::vec::Vec<GLenum>,
    attribute_size_vec: std::vec::Vec<GLint>,
    stride: GLsizei,
) -> (u32, u32) {
    let mut vao = 0;
    let mut vbo = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, size, data, usage);

        let mut offset = 0;
        for i in 0..attribute_type_vec.len() {
            gl::EnableVertexAttribArray(i as u32);
            gl::VertexAttribPointer(
                i as u32,
                attribute_size_vec[i],
                attribute_type_vec[i],
                gl::FALSE,
                stride,
                (offset * mem::size_of::<GLfloat>()) as *const c_void,
            );
            offset += attribute_size_vec[i] as usize;
        }

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    (vao, vbo)
}
