use std::collections::HashMap;
use std::ffi::CString;
use std::ptr::null_mut;
use libsex::bindings::*;
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
        let vert_shader = unsafe { glCreateShader(GL_VERTEX_SHADER) };
        let frag_shader = unsafe { glCreateShader(GL_FRAGMENT_SHADER) };

        // set the source
        unsafe {
            glShaderSource(vert_shader, 1, &vert_source_c.as_ptr(), null_mut());
            glShaderSource(frag_shader, 1, &frag_source_c.as_ptr(), null_mut());
        }

        // compile the shaders
        unsafe {
            glCompileShader(vert_shader);
            glCompileShader(frag_shader);
        }

        // check if the shaders compiled
        let mut status = 0;
        unsafe {
            glGetShaderiv(vert_shader, GL_COMPILE_STATUS, &mut status);
            if status == 0 {
                let mut len = 255;
                glGetShaderiv(vert_shader, GL_INFO_LOG_LENGTH, &mut len);
                let mut log = Vec::with_capacity(len as usize);
                glGetShaderInfoLog(vert_shader, len, null_mut(), log.as_mut_ptr() as *mut GLchar);
                return Err(format!("failed to compile vertex shader: {}", std::str::from_utf8(&log).unwrap()));
            }
            glGetShaderiv(frag_shader, GL_COMPILE_STATUS, &mut status);
            if status == 0 {
                let mut len = 255;
                glGetShaderiv(frag_shader, GL_INFO_LOG_LENGTH, &mut len);
                let mut log = Vec::with_capacity(len as usize);
                glGetShaderInfoLog(frag_shader, len, null_mut(), log.as_mut_ptr() as *mut GLchar);
                return Err(format!("failed to compile fragment shader: {}", std::str::from_utf8(&log).unwrap()));
            }
        }

        // link the shaders
        let shader_program = unsafe { glCreateProgram() };
        unsafe {
            glAttachShader(shader_program, vert_shader);
            glAttachShader(shader_program, frag_shader);
            glLinkProgram(shader_program);
        }

        // check if the shaders linked
        unsafe {
            glGetProgramiv(shader_program, GL_LINK_STATUS, &mut status);
            if status == 0 {
                let mut len = 0;
                glGetProgramiv(shader_program, GL_INFO_LOG_LENGTH, &mut len);
                let mut log = Vec::with_capacity(len as usize);
                glGetProgramInfoLog(shader_program, len, null_mut(), log.as_mut_ptr() as *mut GLchar);
                return Err(format!("failed to link shader program: {}", std::str::from_utf8(&log).unwrap()));
            }
        }

        // clean up
        unsafe {
            glDeleteShader(vert_shader);
            glDeleteShader(frag_shader);
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