pub mod mesh;
pub mod helpers;
pub mod types;
pub mod shader;
pub mod camera;
pub mod keyboard;
pub mod raycasting;
pub mod texture;
pub mod terrain;
pub mod light;

use std::collections::HashMap;
use std::ffi::c_void;
use gfx_maths::{Quaternion, Vec2, Vec3};
use gtk::gdk::{Key, ModifierType};
use libsex::bindings::*;
use crate::renderer::camera::{Camera, CameraMovement};
use crate::renderer::keyboard::KeyboardManager;
use crate::renderer::light::Light;
use crate::renderer::mesh::{Mesh, MeshError};
use crate::renderer::raycasting::Ray;
use crate::renderer::shader::Shader;
use crate::renderer::terrain::Terrain;
use crate::renderer::texture::Texture;
use crate::renderer::types::*;
use crate::worldmachine::{World, WorldMachine};

pub static MAX_LIGHTS: usize = 100;
pub static SHADOW_SIZE: usize = 1024;

pub struct H2eckRenderer {
    pub state: H2eckState,
    pub data_dir: String,
    pub camera: Option<Camera>,
    pub keyboard: KeyboardManager,
    pub last_mouse_position: (f32, f32),
    pub camera_can_move: bool,
    pub current_shader: Option<String>,
    pub shaders: Option<HashMap<String, Shader>>,
    pub meshes: Option<HashMap<String, Mesh>>,
    pub textures: Option<HashMap<String, Texture>>,
    pub terrains: Option<HashMap<String, Terrain>>,
    pub lights: Vec<Light>,
    pub framebuffers: Framebuffers,
    pub selected_entity: isize,
    pub initialised: bool,
    pub shading: bool,
}

pub struct Framebuffers {
    pub original: usize,

    pub postbuffer: usize,
    pub postbuffer_texture: usize,
    pub postbuffer_rbuffer: usize,

    pub depthbuffer: usize,
    pub depthbuffer_texture: usize,

    pub screenquad_vao: usize,
}

pub enum H2eckState {
    Welcome,
}

impl Default for H2eckRenderer {
    fn default() -> Self {
        Self {
            state: H2eckState::Welcome,
            data_dir: String::new(),
            camera: Option::None,
            keyboard: KeyboardManager::default(),
            last_mouse_position: (0.0, 0.0),
            camera_can_move: false,
            current_shader: Option::None,
            shaders: Some(HashMap::new()),
            meshes: Some(HashMap::new()),
            textures: Some(HashMap::new()),
            terrains: Some(HashMap::new()),
            lights: Vec::new(),
            framebuffers: Framebuffers {
                original: 0,
                postbuffer: 0,
                postbuffer_texture: 0,
                postbuffer_rbuffer: 0,
                depthbuffer: 0,
                depthbuffer_texture: 0,
                screenquad_vao: 0,
            },
            selected_entity: -1,
            initialised: false,
            shading: true,
        }
    }
}

impl H2eckRenderer {
    pub fn initialise(&mut self, width: u32, height: u32) {

        let camera = Camera::new(Vec2::new(width as f32, height as f32), 90.0, 0.1, 100.0);
        self.camera = Option::Some(camera);

        // todo! get this from settings
        self.data_dir = String::from("../huskyTech2/base");

        unsafe {
            // get the number of the current framebuffer
            let mut original: i32 = 0;
            glGetIntegerv(GL_FRAMEBUFFER_BINDING, &mut original);
            self.framebuffers.original = original as usize;

            // Configure culling
            glEnable(GL_CULL_FACE);
            glCullFace(GL_FRONT);
            glEnable(GL_DEPTH_TEST);
            glDepthFunc(GL_LESS);

            // configure stencil test
            glEnable(GL_STENCIL_TEST);

            // create the postprocessing framebuffer
            let mut postbuffer = 0;
            glGenFramebuffers(1, &mut postbuffer);
            glBindFramebuffer(GL_FRAMEBUFFER, postbuffer);
            let mut posttexture = 0;
            glGenTextures(1, &mut posttexture);
            glBindTexture(GL_TEXTURE_2D, posttexture);
            glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB as i32, width as i32, height as i32, 0, GL_RGB, GL_UNSIGNED_BYTE, std::ptr::null());
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
            glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, posttexture, 0);
            // create a renderbuffer object for depth and stencil attachment (we won't be sampling these)
            let mut renderbuffer = 0;
            glGenRenderbuffers(1, &mut renderbuffer);
            glBindRenderbuffer(GL_RENDERBUFFER, renderbuffer);
            glRenderbufferStorage(GL_RENDERBUFFER, GL_DEPTH24_STENCIL8, width as i32, height as i32);
            glFramebufferRenderbuffer(GL_FRAMEBUFFER, GL_DEPTH_STENCIL_ATTACHMENT, GL_RENDERBUFFER, renderbuffer);

            // check if framebuffer is complete
            if glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
                panic!("framebuffer is not complete!");
            }
            self.framebuffers.postbuffer = postbuffer as usize;
            self.framebuffers.postbuffer_texture = posttexture as usize;
            self.framebuffers.postbuffer_rbuffer = renderbuffer as usize;

            // create a simple quad that fills the screen
            let mut screenquad_vao = 0;
            glGenVertexArrays(1, &mut screenquad_vao);
            glBindVertexArray(screenquad_vao);
            let mut screenquad_vbo = 0;
            glGenBuffers(1, &mut screenquad_vbo);
            glBindBuffer(GL_ARRAY_BUFFER, screenquad_vbo);
            // just stealing this from the learnopengl.com tutorial (it's a FUCKING QUAD, HOW ORIGINAL CAN IT BE?)
            let quad_vertices: [f32; 30] = [
                // positions        // texture Coords
                -1.0,  1.0, 0.0,    0.0, 1.0,
                -1.0, -1.0, 0.0,    0.0, 0.0,
                1.0, -1.0, 0.0,    1.0, 0.0,

                -1.0,  1.0, 0.0,    0.0, 1.0,
                1.0, -1.0, 0.0,    1.0, 0.0,
                1.0,  1.0, 0.0,    1.0, 1.0,
            ];
            glBufferData(GL_ARRAY_BUFFER, (quad_vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr, quad_vertices.as_ptr() as *const c_void, GL_STATIC_DRAW);
            // as this is such a simple quad, we're not gonna bother with indices
            glEnableVertexAttribArray(0);
            glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE as GLboolean, 5 * std::mem::size_of::<f32>() as i32, std::ptr::null());
            glEnableVertexAttribArray(1);
            glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE as GLboolean, 5 * std::mem::size_of::<f32>() as i32, (3 * std::mem::size_of::<f32>()) as *const c_void);
            self.framebuffers.screenquad_vao = screenquad_vao as usize;

            // create the depth framebuffer
            let mut depthbuffer = 0;
            glGenFramebuffers(1, &mut depthbuffer);
            glBindFramebuffer(GL_FRAMEBUFFER, depthbuffer);
            let mut depthtexture = 0;
            glGenTextures(1, &mut depthtexture);
            glBindTexture(GL_TEXTURE_2D, depthtexture);
            glTexImage2D(GL_TEXTURE_2D, 0, GL_DEPTH_COMPONENT as i32, SHADOW_SIZE as i32, SHADOW_SIZE as i32, 0, GL_DEPTH_COMPONENT, GL_FLOAT, std::ptr::null());
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as i32);
            glFramebufferTexture2D(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_TEXTURE_2D, depthtexture, 0);
            glDrawBuffer(GL_NONE);
            glReadBuffer(GL_NONE);
            if glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
                panic!("framebuffer is not complete (depth buffer)!");
            }

            self.framebuffers.depthbuffer = depthbuffer as usize;
            self.framebuffers.depthbuffer_texture = depthtexture as usize;


            glViewport(0, 0, width as i32, height as i32);
            // make top left corner as origin

            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT);

            // print opengl errors
            let mut error = glGetError();
            while error != GL_NO_ERROR {
                error!("OpenGL error while initialising render subsystem: {}", error);
                error = glGetError();
            }
        }

        Shader::load_shader(self, "postbuffer").expect("failed to load shader (postbuffer)");
        Shader::load_shader(self, "basic").expect("failed to load shader");
        Shader::load_shader(self, "terrain").expect("failed to load shader (terrain)");
        Texture::load_texture("default", "default/default", self, false).expect("failed to load default texture");
        Texture::load_texture("grass1", format!("{}/textures/{}_", self.data_dir,"terrain/grass1").as_str(), self, true).expect("failed to load grass1 texture");
        Texture::load_texture("dirt1", format!("{}/textures/{}_", self.data_dir,"terrain/dirt1").as_str(), self, true).expect("failed to load dirt1 texture");
        Texture::load_texture("rock1", format!("{}/textures/{}_", self.data_dir,"terrain/rock1").as_str(), self, true).expect("failed to load rock1 texture");
        Texture::load_texture("sand1", format!("{}/textures/{}_", self.data_dir,"terrain/sand1").as_str(), self, true).expect("failed to load sand1 texture");

        // some default models that we should load
        self.load_mesh_if_not_already_loaded("ht2").expect("failed to load ht2 model");
    }

    pub fn load_texture_if_not_already_loaded(&mut self, name: &str) -> Result<(), String> {
        if !self.textures.as_ref().unwrap().contains_key(name) {
            Texture::load_texture(name, name, self, false)?;
        }
        Ok(())
    }

    pub fn load_mesh_if_not_already_loaded(&mut self, name: &str) -> Result<(), MeshError> {
        if !self.meshes.as_ref().unwrap().contains_key(name) {
            let mesh = Mesh::new(format!("{}/models/{}.glb", self.data_dir, name).as_str(), name,
                                 &self.shaders.as_mut().unwrap().get("basic").unwrap().clone(), self)?;
            self.meshes.as_mut().unwrap().insert(name.to_string(), mesh);
        }
        Ok(())
    }

    pub fn load_terrain_if_not_already_loaded(&mut self, name: &str) -> Result<(), String> {
        if !self.terrains.as_ref().unwrap().contains_key(name) {
            let terrain = Terrain::new_from_name(name, self)?;
            self.terrains.as_mut().unwrap().insert(name.to_string(), terrain);
        }
        Ok(())
    }

    pub fn process_key(&mut self, key: Key, value: bool) {
        match key {
            Key::w => {
                self.keyboard.forward = value;
            }
            Key::s => {
                self.keyboard.backward = value;
            }
            Key::a => {
                self.keyboard.left = value;
            }
            Key::d => {
                self.keyboard.right = value;
            }
            _ => {}
        };
    }

    pub fn process_inputs(&mut self) {
        let mut vec = Vec3::new(0.0, 0.0, 0.0);
        let scale = 0.1;
        if self.keyboard.forward {
            vec += self.camera.as_mut().unwrap().process_keyboard(CameraMovement::Forward, scale);
        }
        if self.keyboard.backward {
            vec += self.camera.as_mut().unwrap().process_keyboard(CameraMovement::Backward, scale);
        }
        if self.keyboard.left {
            vec += self.camera.as_mut().unwrap().process_keyboard(CameraMovement::Left, scale);
        }
        if self.keyboard.right {
            vec += self.camera.as_mut().unwrap().process_keyboard(CameraMovement::Right, scale);
        }
        self.move_camera(vec);
    }

    pub fn move_camera(&mut self, direction: Vec3) {
        let position = self.camera.as_mut().unwrap().get_position();
        self.camera.as_mut().unwrap().set_position(position + direction);
    }

    pub fn start_rotate_camera(&mut self, mouse_x: f32, mouse_y: f32) {
        self.camera_can_move = true;
    }

    pub fn end_rotate_camera(&mut self, mouse_x: f32, mouse_y: f32) {
        self.camera_can_move = false;
        self.last_mouse_position = (mouse_x, mouse_y);
    }

    pub fn rotate_camera(&mut self, mouse_x_offset: f32, mouse_y_offset: f32) {
        let mouse_x_offset = self.last_mouse_position.0 + mouse_x_offset;
        let mouse_y_offset = self.last_mouse_position.1 + mouse_y_offset;
        let mut camera = self.camera.as_mut().unwrap();
        let mut yaw = helpers::get_quaternion_yaw(camera.get_rotation());
        let mut pitch = helpers::get_quaternion_pitch(camera.get_rotation());
        yaw += -mouse_x_offset;
        pitch += -mouse_y_offset;
        if pitch > 89.0 {
            pitch = 89.0;
        }
        if pitch < -89.0 {
            pitch = -89.0;
        }
        let mut rotation = Quaternion::identity();
        rotation = Quaternion::from_euler_angles_zyx(&Vec3::new(pitch, 0.0, 0.0)) * rotation * Quaternion::from_euler_angles_zyx(&Vec3::new(0.0, yaw, 0.0));
        camera.set_rotation(rotation);
    }

    fn normal_scene_render(&mut self, worldmachine: &mut WorldMachine) {
        unsafe {
            glEnable(GL_CULL_FACE);
            glCullFace(GL_FRONT);
            glEnable(GL_DEPTH_TEST);
            glDepthFunc(GL_LESS);

            // disable gamma correction
            glDisable(GL_FRAMEBUFFER_SRGB);

            // set the clear color to black
            glClearColor(0.1, 0.0, 0.1, 1.0);
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT);

            if let Some(terrains) = self.terrains.clone() {
                // render the terrains
                for terrain in terrains {
                    terrain.1.render(self);
                }
            }

            worldmachine.render(self);
        }
    }

    // should be called upon the render action of our GtkGLArea
    pub fn render(&mut self, worldmachine: &mut WorldMachine) {
        // todo! this is a hack
        if !self.initialised {
            self.initialise(1280, 720);
            debug!("initialised renderer");
            self.initialised = true;
        }

        self.process_inputs();

        let lights = worldmachine.send_lights_to_renderer();
        if let Some(lights) = lights {
            self.lights = lights;
        }

        unsafe {
            glViewport(0, 0, self.camera.as_mut().unwrap().get_window_size().x as GLsizei, self.camera.as_mut().unwrap().get_window_size().y as GLsizei);

            // set framebuffer to the post processing framebuffer
            glBindFramebuffer(GL_FRAMEBUFFER, self.framebuffers.postbuffer as GLuint);

            self.normal_scene_render(worldmachine);

            // set framebuffer to the default framebuffer
            glBindFramebuffer(GL_FRAMEBUFFER, self.framebuffers.original as GLuint);
            glClearColor(1.0, 0.0, 0.1, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);

            if self.current_shader != Some("postbuffer".to_string()) {
                unsafe {
                    glUseProgram(self.shaders.as_mut().unwrap().get("postbuffer").unwrap().program);
                    self.current_shader = Some("postbuffer".to_string());
                }
            }
            // render the post processing framebuffer
            glBindVertexArray(self.framebuffers.screenquad_vao as GLuint);
            glDisable(GL_DEPTH_TEST);

            // enable gamma correction
            glEnable(GL_FRAMEBUFFER_SRGB);

            // make sure that gl doesn't cull the back face of the quad
            glDisable(GL_CULL_FACE);

            // set texture uniform
            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, self.framebuffers.postbuffer_texture as GLuint);
            glUniform1i(glGetUniformLocation(self.shaders.as_mut().unwrap().get("postbuffer").unwrap().program, "u_texture\0".as_ptr() as *const GLchar), 0);
            // draw the screen quad
            glDrawArrays(GL_TRIANGLES, 0, 6);

            // print opengl errors
            let mut error = glGetError();
            while error != GL_NO_ERROR {
                error!("OpenGL error while rendering to postbuffer: {}", error);
                error = glGetError();
            }


            glFlush();
        }
    }
}