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
use glad_gl::gl;
use gtk::gdk::{Key, ModifierType};
use glad_gl::gl::*;
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

        let camera = Camera::new(Vec2::new(width as f32, height as f32), 90.0, 0.1, 10000.0);
        self.camera = Option::Some(camera);

        // todo! get this from settings
        self.data_dir = String::from("../huskyTech2/base");

        unsafe {
            // get the number of the current framebuffer
            let mut original: i32 = 0;
            GetIntegerv(FRAMEBUFFER_BINDING, &mut original);
            self.framebuffers.original = original as usize;

            // Configure culling
            Enable(CULL_FACE);
            CullFace(FRONT);
            Enable(DEPTH_TEST);
            DepthFunc(LESS);

            // configure stencil test
            Enable(STENCIL_TEST);

            // create the postprocessing framebuffer
            let mut postbuffer = 0;
            GenFramebuffers(1, &mut postbuffer);
            BindFramebuffer(FRAMEBUFFER, postbuffer);
            let mut posttexture = 0;
            GenTextures(1, &mut posttexture);
            BindTexture(TEXTURE_2D, posttexture);
            TexImage2D(TEXTURE_2D, 0, RGB as i32, width as i32, height as i32, 0, RGB, UNSIGNED_BYTE, std::ptr::null());
            TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as i32);
            TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as i32);
            FramebufferTexture2D(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D, posttexture, 0);
            // create a renderbuffer object for depth and stencil attachment (we won't be sampling these)
            let mut renderbuffer = 0;
            GenRenderbuffers(1, &mut renderbuffer);
            BindRenderbuffer(RENDERBUFFER, renderbuffer);
            RenderbufferStorage(RENDERBUFFER, DEPTH24_STENCIL8, width as i32, height as i32);
            FramebufferRenderbuffer(FRAMEBUFFER, DEPTH_STENCIL_ATTACHMENT, RENDERBUFFER, renderbuffer);

            // check if framebuffer is complete
            if CheckFramebufferStatus(FRAMEBUFFER) != FRAMEBUFFER_COMPLETE {
                panic!("framebuffer is not complete!");
            }
            self.framebuffers.postbuffer = postbuffer as usize;
            self.framebuffers.postbuffer_texture = posttexture as usize;
            self.framebuffers.postbuffer_rbuffer = renderbuffer as usize;

            // create a simple quad that fills the screen
            let mut screenquad_vao = 0;
            GenVertexArrays(1, &mut screenquad_vao);
            BindVertexArray(screenquad_vao);
            let mut screenquad_vbo = 0;
            GenBuffers(1, &mut screenquad_vbo);
            BindBuffer(ARRAY_BUFFER, screenquad_vbo);
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
            BufferData(ARRAY_BUFFER, (quad_vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr, quad_vertices.as_ptr() as *const c_void, STATIC_DRAW);
            // as this is such a simple quad, we're not gonna bother with indices
            EnableVertexAttribArray(0);
            VertexAttribPointer(0, 3, FLOAT, FALSE as GLboolean, 5 * std::mem::size_of::<f32>() as i32, std::ptr::null());
            EnableVertexAttribArray(1);
            VertexAttribPointer(1, 2, FLOAT, FALSE as GLboolean, 5 * std::mem::size_of::<f32>() as i32, (3 * std::mem::size_of::<f32>()) as *const c_void);
            self.framebuffers.screenquad_vao = screenquad_vao as usize;

            // create the depth framebuffer
            let mut depthbuffer = 0;
            GenFramebuffers(1, &mut depthbuffer);
            BindFramebuffer(FRAMEBUFFER, depthbuffer);
            let mut depthtexture = 0;
            GenTextures(1, &mut depthtexture);
            BindTexture(TEXTURE_2D, depthtexture);
            TexImage2D(TEXTURE_2D, 0, DEPTH_COMPONENT as i32, SHADOW_SIZE as i32, SHADOW_SIZE as i32, 0, DEPTH_COMPONENT, FLOAT, std::ptr::null());
            TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as i32);
            TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as i32);
            TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as i32);
            TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as i32);
            FramebufferTexture2D(FRAMEBUFFER, DEPTH_ATTACHMENT, TEXTURE_2D, depthtexture, 0);
            DrawBuffer(NONE);
            ReadBuffer(NONE);
            if CheckFramebufferStatus(FRAMEBUFFER) != FRAMEBUFFER_COMPLETE {
                panic!("framebuffer is not complete (depth buffer)!");
            }

            self.framebuffers.depthbuffer = depthbuffer as usize;
            self.framebuffers.depthbuffer_texture = depthtexture as usize;

            Enable(BLEND);
            BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);


            Viewport(0, 0, width as i32, height as i32);
            // make top left corner as origin

            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT | STENCIL_BUFFER_BIT);

            // print opengl errors
            let mut error = GetError();
            while error != NO_ERROR {
                error!("OpenGL error while initialising render subsystem: {}", error);
                error = GetError();
            }
        }

        Shader::load_shader(self, "postbuffer").expect("failed to load shader (postbuffer)");
        Shader::load_shader(self, "basic").expect("failed to load shader");
        Shader::load_shader(self, "terrain").expect("failed to load shader (terrain)");
        Shader::load_shader(self, "viz").expect("failed to load shader (viz)");
        Texture::load_texture("default", "default", self, false).expect("failed to load default texture");
        Texture::load_texture("grass1", format!("{}/textures/{}_", self.data_dir,"terrain/grass1").as_str(), self, true).expect("failed to load grass1 texture");
        Texture::load_texture("dirt1", format!("{}/textures/{}_", self.data_dir,"terrain/dirt1").as_str(), self, true).expect("failed to load dirt1 texture");
        Texture::load_texture("rock1", format!("{}/textures/{}_", self.data_dir,"terrain/rock1").as_str(), self, true).expect("failed to load rock1 texture");
        Texture::load_texture("sand1", format!("{}/textures/{}_", self.data_dir,"terrain/sand1").as_str(), self, true).expect("failed to load sand1 texture");

        // some default models that we should load
        self.load_mesh_if_not_already_loaded("ht2").expect("failed to load ht2 model");
        self.load_mesh_if_not_already_loaded("boxviz").expect("failed to load boxviz model");
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
            Enable(CULL_FACE);
            CullFace(FRONT);
            Enable(DEPTH_TEST);
            DepthFunc(LESS);

            // disable gamma correction
            Disable(FRAMEBUFFER_SRGB);

            // set the clear color to black
            ClearColor(0.1, 0.0, 0.1, 1.0);
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT | STENCIL_BUFFER_BIT);

            /*if let Some(terrains) = self.terrains.clone() {
                // render the terrains
                for terrain in terrains {
                    terrain.1.render(self);
                }
            }
             */

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
            Viewport(0, 0, self.camera.as_mut().unwrap().get_window_size().x as GLsizei, self.camera.as_mut().unwrap().get_window_size().y as GLsizei);

            // set framebuffer to the post processing framebuffer
            BindFramebuffer(FRAMEBUFFER, self.framebuffers.postbuffer as GLuint);

            self.normal_scene_render(worldmachine);

            // set framebuffer to the default framebuffer
            BindFramebuffer(FRAMEBUFFER, self.framebuffers.original as GLuint);
            ClearColor(1.0, 0.0, 0.1, 1.0);
            Clear(COLOR_BUFFER_BIT);

            if self.current_shader != Some("postbuffer".to_string()) {
                unsafe {
                    UseProgram(self.shaders.as_mut().unwrap().get("postbuffer").unwrap().program);
                    self.current_shader = Some("postbuffer".to_string());
                }
            }
            // render the post processing framebuffer
            BindVertexArray(self.framebuffers.screenquad_vao as GLuint);
            Disable(DEPTH_TEST);

            // enable gamma correction
            Enable(FRAMEBUFFER_SRGB);

            // make sure that  doesn't cull the back face of the quad
            Disable(CULL_FACE);

            // set texture uniform
            ActiveTexture(TEXTURE0);
            BindTexture(TEXTURE_2D, self.framebuffers.postbuffer_texture as GLuint);
            Uniform1i(GetUniformLocation(self.shaders.as_mut().unwrap().get("postbuffer").unwrap().program, "u_texture\0".as_ptr() as *const GLchar), 0);
            // draw the screen quad
            DrawArrays(TRIANGLES, 0, 6);

            // print opengl errors
            let mut error = GetError();
            while error != NO_ERROR {
                error!("OpenGL error while rendering to postbuffer: {}", error);
                error = GetError();
            }


            Flush();
        }
    }
}