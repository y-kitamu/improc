use std::{ffi::CStr, ptr};
use std::{ffi::CString, io::Read};
use std::{fs::File, path::Path};

use anyhow::Result;
use cgmath::{Array, Matrix, Vector4};
use gl::types::*;

use crate::Matrix4;

pub mod arrow_line_shader;
pub mod image_shader;
pub mod point_shader;
pub mod relation_line_shader;

pub struct UniformVariable<T> {
    pub name: CString, // uniform variable name in glsl program.
    pub value: T,
}

impl<T> UniformVariable<T> {
    pub fn new(name: &str, value: T) -> Self {
        UniformVariable {
            name: CString::new(name).unwrap(),
            value,
        }
    }
}

/// shaderをcompileする.
/// geometry shaderはGL_LINESのみ対応
fn compile_shader(shader_path_stem: &str) -> u32 {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let shader_dir = manifest_dir.join("src").join("shader").join("glsl");

    let vertex_basename = format!("{}.vs", shader_path_stem);
    let fragment_basename = format!("{}.fs", shader_path_stem);
    let geometry_basename = format!("{}.gs", shader_path_stem);
    let vertex = register_shader(
        shader_dir.join(vertex_basename).as_path(),
        gl::VERTEX_SHADER,
    )
    .expect(&format!(
        "Failed to register vertex shader : {}/{}",
        shader_dir.to_str().unwrap(),
        shader_path_stem
    ));
    let fragment = register_shader(
        shader_dir.join(fragment_basename).as_path(),
        gl::FRAGMENT_SHADER,
    )
    .unwrap();
    let geometry = register_shader(
        shader_dir.join(geometry_basename).as_path(),
        gl::GEOMETRY_SHADER,
    );

    unsafe {
        let id = gl::CreateProgram();
        gl::AttachShader(id, vertex);
        gl::AttachShader(id, fragment);
        if let Ok(geo) = geometry {
            gl::AttachShader(id, geo);
            // geometry shader の設定はここ (`gl::AttachShader`と`gl::LInkProgram`の間)でする
            gl::ProgramParameteri(id, gl::GEOMETRY_VERTICES_OUT, 2);
            gl::ProgramParameteri(id, gl::GEOMETRY_INPUT_TYPE, gl::LINES as i32);
            gl::ProgramParameteri(id, gl::GEOMETRY_OUTPUT_TYPE, gl::LINES as i32);
        }
        gl::LinkProgram(id);
        check_compile_errors(id, "PROGRAM");

        gl::DeleteShader(vertex);
        gl::DeleteShader(fragment);
        if let Ok(geo) = geometry {
            gl::DeleteShader(geo);
        }
        id
    }
}

fn register_shader(shader_file_path: &Path, shader_type: GLenum) -> Result<GLuint> {
    let mut file = File::open(shader_file_path)?;
    let mut code = String::new();
    file.read_to_string(&mut code)
        .expect("failed to read vertex shader file");
    let cstr_shader_code = CString::new(code.as_bytes()).unwrap();

    unsafe {
        let shader = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, &cstr_shader_code.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        check_compile_errors(shader, "SHADER");
        Ok(shader)
    }
}

unsafe fn check_compile_errors(shader: u32, type_: &str) {
    let mut success = gl::FALSE as GLint;
    let mut info_log = Vec::with_capacity(1024);
    info_log.set_len(1024 - 1);

    match type_ {
        "PROGRAM" => gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success),
        _ => gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success),
    }
    if success != gl::TRUE as GLint {
        gl::GetShaderInfoLog(
            shader,
            1024,
            ptr::null_mut(),
            info_log.as_mut_ptr() as *mut GLchar,
        );

        let info_log_string = match CStr::from_ptr(info_log.as_ptr()).to_str() {
            Ok(log) => log,
            Err(vec) => {
                panic!("failed to convert to compilation log from buffer : {}", vec)
            }
        };
        println!(
            "failed to compile or link shader code : type = {}, log ={}",
            type_, info_log_string
        );
    }
}

unsafe fn set_vec4(shader_id: u32, u_var: &UniformVariable<Vector4<f32>>) {
    // println!("shader_id : {}, value = {:?}", shader_id, u_var.value);
    gl::Uniform4fv(
        gl::GetUniformLocation(shader_id, u_var.name.as_ptr()),
        1,
        u_var.value.as_ptr(),
    );
}

unsafe fn set_mat4(shader_id: u32, u_var: &UniformVariable<Matrix4>) {
    gl::UniformMatrix4fv(
        gl::GetUniformLocation(shader_id, u_var.name.as_ptr()),
        1,
        gl::FALSE,
        u_var.value.as_ptr(),
    );
}

unsafe fn set_mat4_array(shader_id: u32, u_var: &UniformVariable<Vec<Matrix4>>) {
    gl::UniformMatrix4fv(
        gl::GetUniformLocation(shader_id, u_var.name.as_ptr()),
        u_var.value.len() as i32,
        gl::FALSE,
        u_var.value[0].as_ptr(),
    )
}

unsafe fn set_float(shader_id: u32, u_var: &UniformVariable<f32>) {
    gl::Uniform1f(
        gl::GetUniformLocation(shader_id, u_var.name.as_ptr()),
        u_var.value,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_variable() {
        let uval = UniformVariable::new("test", vec![1u8; 3]);
        assert_eq!(uval.value[0], 1);
        assert_eq!(uval.value[1], 1);
        assert_eq!(uval.value[2], 1);
        assert_eq!(uval.value.len(), 3);
        assert_eq!(uval.name, CString::new("test").unwrap());
    }
}
