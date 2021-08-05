use std::mem;
use std::os::raw::c_void;

use gl::types::{GLenum, GLfloat, GLint, GLsizei, GLsizeiptr};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Vertex {
    vao: u32,
    vbo: u32,
    vertex_num: i32,
}

// 画像描画用のvertex作成
pub fn create_simple_vertex() -> Vertex {
    #[rustfmt::skip]
    let buf_array: [f32; 30] = [
        // 0
        -1.0, -1.0, 1.0, 0.0, 0.0,
        -1.0, 1.0, 1.0, 0.0, 1.0,
        1.0, 1.0, 1.0, 1.0, 1.0,
        -1.0, -1.0, 1.0, 0.0, 0.0,
        1.0, 1.0, 1.0, 1.0, 1.0,
        1.0, -1.0, 1.0, 1.0, 0.0,
    ];
    Vertex::new(
        (30 * mem::size_of::<GLfloat>()) as GLsizeiptr,
        buf_array.as_ptr() as *const c_void,
        gl::STATIC_DRAW,
        vec![gl::FLOAT, gl::FLOAT],
        vec![3, 2],
        (5 * mem::size_of::<GLfloat>()) as GLsizei,
        6,
    )
}

impl Vertex {
    pub fn new(
        size: GLsizeiptr,
        data: *const c_void,
        usage: GLenum,
        attribute_type_vec: std::vec::Vec<GLenum>,
        attribute_size_vec: std::vec::Vec<GLint>,
        stride: GLsizei,
        vertex_num: i32,
    ) -> Vertex {
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

        Vertex {
            vao,
            vbo,
            vertex_num,
        }
    }

    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_num);
            gl::BindVertexArray(0);
        }
    }
}
