pub mod mesh;
pub mod helpers;
pub mod types;
pub mod shader;
pub mod camera;
pub mod keyboard;
pub mod raycasting;
pub mod texture;
pub mod terrain;

use std::collections::HashMap;
use std::ffi::c_void;
use dae_parser::Document;
use gfx_maths::{Quaternion, Vec2, Vec3};
use gtk::gdk::{Key, ModifierType};
use libsex::bindings::*;
use crate::renderer::camera::{Camera, CameraMovement};
use crate::renderer::keyboard::KeyboardManager;
use crate::renderer::mesh::Mesh;
use crate::renderer::raycasting::Ray;
use crate::renderer::shader::Shader;
use crate::renderer::terrain::Terrain;
use crate::renderer::texture::Texture;
use crate::renderer::types::*;
use crate::worldmachine::{World, WorldMachine};

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
    pub selection_framebuffer: isize,
    pub selection_texture: isize,
    pub selected_entity: isize,
    pub initialised: bool,
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
            selection_framebuffer: -1,
            selection_texture: -1,
            selected_entity: -1,
            initialised: false,
        }
    }
}

impl H2eckRenderer {
    pub fn initialise(&mut self, width: u32, height: u32) {

        let camera = Camera::new(Vec2::new(width as f32, height as f32), 90.0, 0.1, 100.0);
        self.camera = Option::Some(camera);

        // todo! get this from settings
        self.data_dir = String::from("../huskyTech2/base");

        //self.create_selection_framebuffer(self.camera.as_ref().unwrap().get_window_size());

        unsafe {
            // Configure culling
            glEnable(GL_CULL_FACE);
            glCullFace(GL_FRONT);
            glEnable(GL_DEPTH_TEST);
            glDepthFunc(GL_LESS);

            glViewport(0, 0, width as i32, height as i32);
            // make top left corner as origin

            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            // print opengl errors
            let mut error = glGetError();
            while error != GL_NO_ERROR {
                error!("OpenGL error while initialising render subsystem: {}", error);
                error = glGetError();
            }
        }

        Shader::load_shader(self, "basic").expect("failed to load shader");
        Shader::load_shader(self, "terrain").expect("failed to load shader (terrain)");
        Texture::load_texture("default", "default/default", self).expect("failed to load default texture");
        Texture::load_texture("grass1", "terrain/grass1", self).expect("failed to load grass1 texture");
        Texture::load_texture("dirt1", "terrain/dirt1", self).expect("failed to load dirt1 texture");
        Texture::load_texture("rock1", "terrain/rock1", self).expect("failed to load rock1 texture");
        Texture::load_texture("sand1", "terrain/sand1", self).expect("failed to load sand1 texture");

        let terrain = Terrain::new_from_name("ll_main", self).expect("failed to load terrain");

        self.terrains = Some(HashMap::new());
        self.terrains.as_mut().unwrap().insert("ll_main".to_string(), terrain);

        let ht2_document = Document::from_file("internal/models/ht2.dae").expect("failed to load ht2.dae");
        let mut ht2_mesh =
            Mesh::new(ht2_document, "ht2-mesh",
                 &self.shaders.as_mut().unwrap().get("basic").unwrap().clone(), self)
            .expect("failed to create ht2 mesh");
        //ht2_mesh.position = Vec3::new(0.0, 0.25, 4.0);
        self.meshes.as_mut().unwrap().insert("ht2".to_string(), ht2_mesh);
    }

    fn create_selection_framebuffer(&mut self, window_size: Vec2) {
        unsafe {
            let mut framebuffer = 0;

            glGenFramebuffers(1, &mut framebuffer);
            glBindFramebuffer(GL_FRAMEBUFFER, framebuffer);

            let mut texture = 0;

            let mut depth_texture = 0;

            glGenTextures(1, &mut texture);
            glBindTexture(GL_TEXTURE_2D, texture);
            glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB32UI as i32, window_size.x as i32, window_size.y as i32, 0, GL_RGB_INTEGER, GL_UNSIGNED_INT, std::ptr::null());
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST as i32);
            glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, texture, 0);

            glGenTextures(1, &mut depth_texture);
            glBindTexture(GL_TEXTURE_2D, depth_texture);
            glTexImage2D(GL_TEXTURE_2D, 0, GL_DEPTH_COMPONENT32F as i32, window_size.x as i32, window_size.y as i32, 0, GL_DEPTH_COMPONENT, GL_FLOAT, std::ptr::null());
            glFramebufferTexture2D(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_TEXTURE_2D, depth_texture, 0);

            if glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
                error!("failed to create selection framebuffer");
            }

            self.selection_framebuffer = framebuffer as isize;
            self.selection_texture = texture as isize;

            glBindFramebuffer(GL_FRAMEBUFFER, 0);
            glBindTexture(GL_TEXTURE_2D, 0);
        }
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

    pub fn get_id_from_pixel(&self, position: Vec2) -> Option<u32> {
        unsafe {
            glBindFramebuffer(GL_FRAMEBUFFER, self.selection_framebuffer as u32);
            glReadBuffer(GL_COLOR_ATTACHMENT0);
            let mut pixel = [0, 0, 0];
            glReadPixels(position.x as i32, position.y as i32, 1, 1, GL_RGB_INTEGER, GL_UNSIGNED_INT, pixel.as_mut_ptr() as *mut c_void);
            glReadBuffer(GL_NONE);
            glBindFramebuffer(GL_FRAMEBUFFER, 0);
            if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 {
                return Option::None;
            }
            return Option::Some(pixel[0] as u32);
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

        unsafe {
            // set the clear color to black
            glClearColor(0.1, 0.0, 0.1, 1.0);
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            if let Some(terrains) = self.terrains.clone() {
                // render the terrains
                for terrain in terrains {
                    terrain.1.render(self);
                }
            }

            worldmachine.render(self);

            glFlush();
        }
    }
}