use std::mem;
use std::os::raw::c_void;

use gl::types::{GLenum, GLfloat, GLint, GLsizei, GLsizeiptr};

use crate::{
    shader::{Shader, UniformVariable},
    Mat4,
};

pub mod arrows;
pub mod image;
pub mod lines;
pub mod points;
pub mod screen;

#[derive(PartialEq)]
pub enum DrawableType {
    Arrows,
    Image,
    Line,
    Points,
    Screen,
}

pub trait Drawable {
    /// Return drawable type
    fn get_drawable_type(&self) -> DrawableType;
    /// Return number of vertex in the object.
    fn get_vertex_num(&self) -> u32;
    /// Return draw type passed to `gl::DrawArrays'.
    fn get_draw_type(&self) -> gl::types::GLenum;
    /// Return model matrix.
    fn get_model_mat(&mut self) -> Mat4;
    /// Return mutable `Shader` of the object.
    fn get_mut_shader(&mut self) -> &mut Box<dyn Shader>;
    /// Return mutable `Drawable` associated with the object.
    fn get_associated_drawables(&mut self) -> &Vec<Box<dyn Drawable>>;
    fn get_mut_associated_drawables(&mut self) -> &mut Vec<Box<dyn Drawable>>;
    /// Return true if the object is a drawing target else false.
    fn is_draw(&self) -> bool;
    fn set_is_draw(&mut self, flag: bool);
    /// If gl vertex array object is registered, return vao id, else 0.
    fn get_vao(&self) -> u32;
    /// If gl texture is registered, return texture id, else 0.
    fn get_texture_id(&self) -> u32 {
        0
    }
    fn build(&mut self) {
        for obj in self.get_mut_associated_drawables() {
            obj.build();
        }
    }
    /// Draw the object.
    fn draw(&mut self, view_mat: &UniformVariable<Mat4>, proj_mat: &UniformVariable<Mat4>) {
        if !self.is_draw() {
            return;
        }
        let shader = self.get_mut_shader();
        shader.set_uniform_variables(view_mat, proj_mat);
        unsafe {
            gl::UseProgram(shader.get_id());
            gl::BindTexture(gl::TEXTURE_2D, self.get_texture_id());
            gl::BindVertexArray(self.get_vao());
            gl::DrawArrays(self.get_draw_type(), 0, self.get_vertex_num() as i32);
            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::UseProgram(0);
        }
        for obj in self.get_mut_associated_drawables() {
            obj.draw(view_mat, proj_mat);
        }
    }
    /// Draw imgui widgets
    fn draw_imgui(&mut self, ui: &imgui::Ui) {
        self.get_mut_shader().draw_imgui(ui);
    }
    /// Update variable
    fn update(&mut self, view_mat: &UniformVariable<Mat4>, proj_mat: &UniformVariable<Mat4>) {
        self.get_mut_shader()
            .set_uniform_variables(view_mat, proj_mat);
        for obj in self.get_mut_associated_drawables() {
            obj.update(view_mat, proj_mat);
        }
    }
}

trait PointLike {
    fn to_vec(&self) -> Vec<f32>;
}

/// texture描画用のvertex作成
/// 返り値は(vao id, vbo id, n_vertex)
pub fn create_simple_vertex() -> (u32, u32, u32) {
    #[rustfmt::skip]
    let buf_array: [f32; 30] = [
        -1.0, -1.0, 1.0, 0.0, 0.0,
        -1.0, 1.0, 1.0, 0.0, 1.0,
        1.0, 1.0, 1.0, 1.0, 1.0,
        -1.0, -1.0, 1.0, 0.0, 0.0,
        1.0, 1.0, 1.0, 1.0, 1.0,
        1.0, -1.0, 1.0, 1.0, 0.0,
    ];
    let (vao, vbo) = register_primitive(
        &buf_array,
        gl::STATIC_DRAW,
        vec![gl::FLOAT, gl::FLOAT],
        vec![3, 2],
        (5 * mem::size_of::<GLfloat>()) as GLsizei,
    );
    (vao, vbo, 6)
}

fn build_pointlike_cloud<T>(
    arr: &Vec<T>,
    attrib_type: Vec<GLenum>,
    attrib_size: Vec<GLint>,
) -> (u32, u32, u32)
where
    T: PointLike,
{
    if arr.len() == 0 {
        return (0, 0, 0);
    }
    let n_vertex_per_point = arr[0].to_vec().len();
    let buf_array = arr.iter().flat_map(|p| p.to_vec()).collect::<Vec<f32>>();
    let (vao, vbo) = register_primitive(
        &buf_array,
        gl::STATIC_DRAW,
        attrib_type,
        attrib_size,
        (n_vertex_per_point * mem::size_of::<GLfloat>()) as GLsizei,
    );
    (vao, vbo, (buf_array.len() / n_vertex_per_point) as u32)
}

/// OpenGLのprimitiveを作成、vao, vboを返す
fn register_primitive(
    buf_array: &[f32],
    usage: GLenum,
    attribute_type_vec: Vec<GLenum>,
    attribute_size_vec: Vec<GLint>,
    stride: GLsizei,
) -> (u32, u32) {
    let size = (buf_array.len() as usize * mem::size_of::<GLfloat>()) as GLsizeiptr;
    let data = buf_array.as_ptr() as *const c_void;
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
