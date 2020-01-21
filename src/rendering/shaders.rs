use crate::gl;
use std::collections::HashMap;
use std::ffi::CStr;
use std::str::FromStr;

static VS_SHADER_SRC: &'static [u8] = b"
#version 450

uniform mat4 u_projMatrix;

layout(location=0) in vec3 vert_pos;
layout(location=1) in vec3 vert_normal;

out vec2 tex_coords;

void main()
{
    tex_coords = vec2(vert_normal.x, 0.0);
    gl_Position = u_projMatrix * vec4(vert_pos, 1.0);
}\0";

static FS_SHADER_SRC: &'static [u8] = b"
#version 450
layout(binding=0) uniform sampler2D tex_sampler;

in vec2 tex_coords;

out vec4 out_col;

void main()
{
    vec4 color = texture(tex_sampler, tex_coords);
    out_col = vec4(color.rgb, 1.0);
}\0";

pub struct Program {
    program_name: u32,
    uniforms: HashMap<String, i32>,
}

impl Program {
    pub fn new() -> Self {
        let program = create_shader_program();
        let mut max_name_len = 0i32;
        let mut uniform_count = 0i32;
        let mut uniform_type = 0u32;
        let mut length = 0i32;
        let mut count = 0i32;
        let mut uniform_map = std::collections::HashMap::<String, i32>::new();
        unsafe {
            gl::GetProgramiv(program, gl::ACTIVE_UNIFORMS, &mut uniform_count);
            gl::GetProgramiv(program, gl::ACTIVE_UNIFORM_MAX_LENGTH, &mut max_name_len);
            println!("Got max name len: {}", max_name_len);
            let mut uniform_name: Vec<i8> = vec![];
            uniform_name.resize(max_name_len as usize, 0);
            for i in 0..uniform_count as u32 {
                gl::GetActiveUniform(
                    program,
                    i,
                    max_name_len,
                    &mut length,
                    &mut count,
                    &mut uniform_type,
                    uniform_name.as_mut_ptr(),
                );
                let location = gl::GetUniformLocation(program, uniform_name.as_ptr());

                let uniform_str =
                    String::from_str(CStr::from_ptr(uniform_name.as_ptr()).to_str().unwrap())
                        .unwrap();

                uniform_map.insert(uniform_str, location);
            }
            dbg!(&uniform_map);
        }
        Self {
            program_name: program,
            uniforms: uniform_map,
        }
    }

    pub unsafe fn bind(&self) {
        gl::UseProgram(self.program_name);
    }

    pub unsafe fn uniform_mat(&self, uniform_str: &String, matrix: &mikpe_math::Mat4) {
        if let Some(uniform_location) = self.uniforms.get(uniform_str) {
            gl::ProgramUniformMatrix4fv(
                self.program_name,
                *uniform_location,
                1,
                0,
                &matrix[0][0] as *const _,
            );
        } else {
            println!("Could not find uniform {}!", uniform_str);
        }
    }
}
fn make_shader(shader_type: gl::types::GLenum, shader_src: &[u8]) -> u32 {
    unsafe {
        let shader_id = gl::CreateShader(shader_type);
        //src to CStr
        let src_cstr = CStr::from_bytes_with_nul(shader_src).unwrap();
        let shader_len = src_cstr.to_bytes().len() as i32;
        gl::ShaderSource(
            shader_id,
            1,
            &src_cstr.as_ptr() as *const *const _,
            &shader_len as *const _,
        );
        gl::CompileShader(shader_id);
        let mut success = 0;
        gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);
        if success <= 0 {
            let mut info_log = [0i8; 512];
            let mut placeholder = 0;
            gl::GetShaderInfoLog(shader_id, 512, &mut placeholder, info_log.as_mut_ptr());
            let cstrinfo = CStr::from_ptr(info_log.as_ptr());
            println!("Shader compilation error: \n{}", cstrinfo.to_str().unwrap());
        };
        shader_id
    }
}

fn link_program(vs_shader: u32, fs_shader: u32) -> u32 {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs_shader);
        gl::AttachShader(program, fs_shader);
        gl::LinkProgram(program);
        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        let mut placeholder = 0;
        if success <= 0 {
            let mut info_log = [0i8; 512];
            gl::GetProgramInfoLog(program, 512, &mut placeholder, info_log.as_mut_ptr());
            let cstrinfo = CStr::from_ptr(info_log.as_ptr());
            println!("Program link error: \n{}", cstrinfo.to_str().unwrap());
        }
        program
    }
}

pub fn create_shader_program() -> u32 {
    let vs_shader = make_shader(gl::VERTEX_SHADER, VS_SHADER_SRC);
    let fs_shader = make_shader(gl::FRAGMENT_SHADER, FS_SHADER_SRC);
    let program = link_program(vs_shader, fs_shader);
    program
}
