use std::{collections::HashMap, path::Path};

use cgmath::Array;
use cgmath::Matrix;
use gl;
use gl::types::*;

use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;
use std::ptr;
use std::str;

#[allow(dead_code)]
type Vector3 = cgmath::Vector3<f32>;
#[allow(dead_code)]
type Matrix4 = cgmath::Matrix4<f32>;

pub struct Shader {
    pub id: u32,
}

const SHADER_LIST: [&str; 1] = ["default"];

pub fn load_shaders() -> HashMap<String, Shader> {
    let mut shader_map = HashMap::<String, Shader>::new();
    SHADER_LIST.iter().for_each(|key| {
        let shader = Shader::new(&key);
        shader_map.insert(key.to_string(), shader);
    });
    shader_map
}

#[allow(dead_code)]
impl Shader {
    pub fn new(shader_path_stem: &str) -> Shader {
        let cur_file = Path::new(file!());
        let cur_dir = cur_file.parent().unwrap();
        let shader_dir = cur_dir.join("shader");

        let mut shader = Shader { id: 0 };

        let vertex_basename = format!("{}.vs", shader_path_stem);
        let fragment_basename = format!("{}.fs", shader_path_stem);
        let vertex = shader.register_shader(
            shader_dir.join(vertex_basename).as_path(),
            gl::VERTEX_SHADER,
        );
        let fragment = shader.register_shader(
            shader_dir.join(fragment_basename).as_path(),
            gl::FRAGMENT_SHADER,
        );

        unsafe {
            let id = gl::CreateProgram();
            gl::AttachShader(id, vertex);
            gl::AttachShader(id, fragment);
            gl::LinkProgram(id);
            shader.check_compile_errors(id, "PROGRAM");

            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);

            shader.id = id;
        }
        println!(
            "finish create shader  : key = {}, id = {}",
            shader_path_stem, shader.id
        );
        shader
    }

    fn register_shader(&self, shader_file_path: &Path, shader_type: GLenum) -> GLuint {
        let mut file = File::open(shader_file_path)
            .unwrap_or_else(|_| panic!("failed to open file : {:?}", shader_file_path));
        let mut code = String::new();
        file.read_to_string(&mut code)
            .expect("failed to read vertex shader file");
        let cstr_shader_code = CString::new(code.as_bytes()).unwrap();

        unsafe {
            let shader = gl::CreateShader(shader_type);
            gl::ShaderSource(shader, 1, &cstr_shader_code.as_ptr(), ptr::null());
            gl::CompileShader(shader);
            self.check_compile_errors(shader, "SHADER");
            shader
        }
    }

    unsafe fn check_compile_errors(&self, shader: u32, type_: &str) {
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

    pub fn get_shader_id(&self) -> u32 {
        self.id
    }

    pub unsafe fn set_bool(&self, name: &CStr, value: bool) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value as i32);
    }

    pub unsafe fn set_int(&self, name: &CStr, value: i32) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value);
    }

    pub unsafe fn set_float(&self, name: &CStr, value: f32) {
        gl::Uniform1f(gl::GetUniformLocation(self.id, name.as_ptr()), value);
    }

    pub unsafe fn set_vector3(&self, name: &CStr, value: &Vector3) {
        gl::Uniform3fv(
            gl::GetUniformLocation(self.id, name.as_ptr()),
            1,
            value.as_ptr(),
        );
    }

    pub unsafe fn set_vec3(&self, name: &CStr, x: f32, y: f32, z: f32) {
        gl::Uniform3f(gl::GetUniformLocation(self.id, name.as_ptr()), x, y, z);
    }

    pub unsafe fn set_mat4(&self, name: &CStr, mat: &Matrix4) {
        gl::UniformMatrix4fv(
            gl::GetUniformLocation(self.id, name.as_ptr()),
            1,
            gl::FALSE,
            mat.as_ptr(),
        );
    }
}
