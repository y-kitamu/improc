use std::{collections::HashMap, path::Path};

use c_str_macro::c_str;
use cgmath::{Matrix, One, Transform, Vector4};
use gl::types::*;
use gl::{self, GetActiveUniformName};
use sdl2::mouse::{MouseButton, MouseState, MouseWheelDirection};

use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;
use std::ptr;
use std::str;

#[allow(dead_code)]
type Vector3 = cgmath::Vector3<f32>;
#[allow(dead_code)]
type Matrix4 = cgmath::Matrix4<f32>;

const SHADER_LIST: [&str; 1] = ["default"];

pub fn load_shaders() -> HashMap<String, Shader> {
    let mut shader_map = HashMap::<String, Shader>::new();
    SHADER_LIST.iter().for_each(|key| {
        let shader = Shader::new(&key);
        shader_map.insert(key.to_string(), shader);
    });
    shader_map
}

fn compile_shader(shader_path_stem: &str) -> u32 {
    let cur_file = Path::new(file!());
    let cur_dir = cur_file.parent().unwrap();
    let shader_dir = cur_dir.join("shader");

    let vertex_basename = format!("{}.vs", shader_path_stem);
    let fragment_basename = format!("{}.fs", shader_path_stem);
    let vertex = register_shader(
        shader_dir.join(vertex_basename).as_path(),
        gl::VERTEX_SHADER,
    );
    let fragment = register_shader(
        shader_dir.join(fragment_basename).as_path(),
        gl::FRAGMENT_SHADER,
    );

    unsafe {
        let id = gl::CreateProgram();
        gl::AttachShader(id, vertex);
        gl::AttachShader(id, fragment);
        gl::LinkProgram(id);
        check_compile_errors(id, "PROGRAM");

        gl::DeleteShader(vertex);
        gl::DeleteShader(fragment);

        id
    }
}

fn register_shader(shader_file_path: &Path, shader_type: GLenum) -> GLuint {
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
        check_compile_errors(shader, "SHADER");
        shader
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

pub struct UniformVariable<T> {
    pub name: CString, // uniform variable name in glsl program.
    pub value: T,
}

pub struct Shader {
    id: u32,
    pub model_mat: UniformVariable<Matrix4>,
    pub view_mat: UniformVariable<Matrix4>,
    pub projection_mat: UniformVariable<Matrix4>,
    pub point_size: UniformVariable<f32>,
    is_dragging: bool, // 画像をdrag中かどうか
}

#[allow(dead_code)]
impl Shader {
    pub fn new(shader_path_stem: &str) -> Self {
        let id = compile_shader(shader_path_stem);
        let model_mat = UniformVariable {
            name: CString::new("uModel").unwrap(),
            value: Matrix4::one(),
        };
        let view_mat = UniformVariable {
            name: CString::new("uView").unwrap(),
            value: Matrix4::one(),
        };
        let projection_mat = UniformVariable {
            name: CString::new("uProjection").unwrap(),
            value: Matrix4::one(),
        };
        let point_size = UniformVariable {
            name: CString::new("uPointSize").unwrap(),
            value: 10.0f32,
        };
        Shader {
            id,
            model_mat,
            view_mat,
            projection_mat,
            point_size,
            is_dragging: false,
        }
    }

    /// 元画像のaspect ratioが保存されるようにmodel matrixを調整する
    pub fn adjust_aspect_ratio(
        &mut self,
        image_width: u32,
        image_height: u32,
        screen_width: u32,
        screen_height: u32,
    ) {
        let aspect_ratio =
            image_height as f32 * screen_width as f32 / (image_width as f32 * screen_height as f32);
        match aspect_ratio < 1.0f32 {
            true => {
                self.model_mat.value[1][1] = self.model_mat.value[0][0] * aspect_ratio;
            }
            false => {
                self.model_mat.value[0][0] = self.model_mat.value[1][1] / aspect_ratio;
            }
        }
    }

    /// 画像を拡大縮小する
    pub fn on_mouse_wheel_event(
        &mut self,
        timestamp: &u32,
        window_id: &u32,
        which: &u32,
        x: &i32,
        y: &i32,
        direction: &MouseWheelDirection,
    ) {
        let mut scale = 1.0f32 + *y as f32 / 10.0f32;
        if *direction == MouseWheelDirection::Flipped {
            scale = 1.0f32 / scale;
        }
        self.model_mat.value[0][0] *= scale;
        self.model_mat.value[1][1] *= scale;
    }

    ///
    pub fn on_mouse_motion_event(
        &mut self,
        timestamp: &u32,
        window_id: &u32,
        which: &u32,
        mousestate: &MouseState,
        x: &i32,
        y: &i32,
        xrel: f32,
        yrel: f32,
    ) {
        if self.is_dragging {
            self.model_mat.value[3][0] += xrel;
            self.model_mat.value[3][1] += yrel;
        }
    }

    /// mouseが画像をクリックしたか判定する
    pub fn on_mouse_button_down(
        &mut self,
        timestamp: &u32,
        window_id: &u32,
        which: &u32,
        mouse_btn: &MouseButton,
        clicks: &u8,
        x: f32, // -1.0 to 1.0
        y: f32, // -1.0 to 1.0
    ) {
        let nx = x - self.model_mat.value[3][0];
        let ny = y - self.model_mat.value[3][1];
        self.is_dragging =
            (nx.abs() <= self.model_mat.value[0][0]) && (ny.abs() <= self.model_mat.value[1][1]);
    }

    pub fn on_mouse_button_up(
        &mut self,
        timestamp: &u32,
        window_id: &u32,
        which: &u32,
        mouse_btn: &MouseButton,
        clicks: &u8,
        x: &i32, // -1.0 to 1.0
        y: &i32, // -1.0 to 1.0
    ) {
        self.is_dragging = false;
    }

    /// glslのuniform変数をセットする
    pub fn set_uniform_variables(&self, id: u32, with_pts: bool) {
        unsafe {
            set_mat4(id, &self.model_mat);
            set_mat4(id, &self.view_mat);
            set_mat4(id, &self.projection_mat);
            if with_pts {
                // println!("point size = {}", self.point_size.value);
                set_float(id, &self.point_size);
            }
        }
    }

    pub fn get_shader_id(&self) -> u32 {
        self.id
    }
}

unsafe fn set_mat4(shader_id: u32, u_var: &UniformVariable<Matrix4>) {
    gl::UniformMatrix4fv(
        gl::GetUniformLocation(shader_id, u_var.name.as_ptr()),
        1,
        gl::FALSE,
        u_var.value.as_ptr(),
    );
}

unsafe fn set_float(shader_id: u32, u_var: &UniformVariable<f32>) {
    gl::Uniform1f(
        gl::GetUniformLocation(shader_id, u_var.name.as_ptr()),
        u_var.value,
    )
}
