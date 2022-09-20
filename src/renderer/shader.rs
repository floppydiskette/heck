use std::collections::HashMap;
use std::ffi::CString;
use std::ptr::null_mut;
use glad_gl::gl::*;
use crate::renderer::{H2eckRenderer, helpers};

#[derive(Clone)]
pub struct Shader {
    pub name: String,
    pub program: GLuint,
}

impl Shader {
    pub fn load_shader(renderer: &mut H2eckRenderer, shader_name: &str) -> Result<(), String> {
        // read the files
        let vert_source = helpers::load_string_from_file(format!("internal/shaders/{}.vert", shader_name)).expect("failed to load vertex shader");
        let frag_source = helpers::load_string_from_file(format!("internal/shaders/{}.frag", shader_name)).expect("failed to load fragment shader");

        // convert strings to c strings
        let vert_source_c = CString::new(vert_source).unwrap();
        let frag_source_c = CString::new(frag_source).unwrap();

        // create the shaders
        let vert_shader = unsafe { CreateShader(VERTEX_SHADER) };
        let frag_shader = unsafe { CreateShader(FRAGMENT_SHADER) };

        // set the source
        unsafe {
            ShaderSource(vert_shader, 1, &vert_source_c.as_ptr(), null_mut());
            ShaderSource(frag_shader, 1, &frag_source_c.as_ptr(), null_mut());
        }

        // compile the shaders
        unsafe {
            CompileShader(vert_shader);
            CompileShader(frag_shader);
        }

        // check if the shaders compiled
        let mut status = 0;
        unsafe {
            GetShaderiv(vert_shader, COMPILE_STATUS, &mut status);
            if status == 0 {
                let mut len = 255;
                GetShaderiv(vert_shader, INFO_LOG_LENGTH, &mut len);
                let log = vec![0; len as usize + 1];
                let log_c = CString::from_vec_unchecked(log);
                let log_p = log_c.into_raw();
                GetShaderInfoLog(vert_shader, len, null_mut(), log_p);
                return Err(format!("failed to compile vertex shader: {}", CString::from_raw(log_p).to_string_lossy()));
            }
            GetShaderiv(frag_shader, COMPILE_STATUS, &mut status);
            if status == 0 {
                let mut len = 255;
                GetShaderiv(frag_shader, INFO_LOG_LENGTH, &mut len);
                let log = vec![0; len as usize + 1];
                let log_c = CString::from_vec_unchecked(log);
                let log_p = log_c.into_raw();
                GetShaderInfoLog(frag_shader, len, null_mut(), log_p);
                return Err(format!("failed to compile fragment shader: {}", CString::from_raw(log_p).to_string_lossy()));
            }
        }

        // link the shaders
        let shader_program = unsafe { CreateProgram() };
        unsafe {
            AttachShader(shader_program, vert_shader);
            AttachShader(shader_program, frag_shader);
            LinkProgram(shader_program);
        }

        // check if the shaders linked
        unsafe {
            GetProgramiv(shader_program, LINK_STATUS, &mut status);
            if status == 0 {
                let mut len = 0;
                GetProgramiv(shader_program, INFO_LOG_LENGTH, &mut len);
                let log = vec![0; len as usize + 1];
                let log_c = CString::from_vec_unchecked(log);
                let log_p = log_c.into_raw();
                GetProgramInfoLog(shader_program, len, null_mut(), log_p);
                return Err(format!("failed to link shader program: {}", CString::from_raw(log_p).to_string_lossy()));
            }
        }

        // clean up
        unsafe {
            DeleteShader(vert_shader);
            DeleteShader(frag_shader);
        }

        // add shader to list
        if renderer.shaders.is_none() {
            renderer.shaders = Option::Some(HashMap::new());
        }
        renderer.shaders.as_mut().unwrap().insert(shader_name.to_string(), Shader {
            name: shader_name.to_string(),
            program: shader_program,
        });

        // return the index of the shader
        Ok(())
    }
}