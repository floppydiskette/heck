pub mod mesh;
pub mod helpers;
pub mod types;
pub mod shader;
pub mod camera;
pub mod keyboard;

use std::collections::HashMap;
use dae_parser::Document;
use gfx_maths::{Quaternion, Vec2, Vec3};
use gtk::gdk::{Key, ModifierType};
use libsex::bindings::*;
use crate::renderer::camera::{Camera, CameraMovement};
use crate::renderer::keyboard::KeyboardManager;
use crate::renderer::mesh::Mesh;
use crate::renderer::shader::Shader;
use crate::renderer::types::*;
use crate::worldmachine::{World, WorldMachine};

pub struct H2eckRenderer {
    pub state: H2eckState,
    pub camera: Option<Camera>,
    pub keyboard: KeyboardManager,
    pub first_mouse_pos: (f32, f32), // offset from 0,0
    pub camera_can_move: bool,
    pub current_shader: Option<String>,
    pub shaders: Option<HashMap<String, Shader>>,
    pub meshes: Option<HashMap<String, Mesh>>,
    pub initialised: bool,
}

pub enum H2eckState {
    Welcome,
}

impl Default for H2eckRenderer {
    fn default() -> Self {
        Self {
            state: H2eckState::Welcome,
            camera: Option::None,
            keyboard: KeyboardManager::default(),
            first_mouse_pos: (0.0, 0.0),
            camera_can_move: false,
            current_shader: Option::None,
            shaders: Some(HashMap::new()),
            meshes: Some(HashMap::new()),
            initialised: false,
        }
    }
}

impl H2eckRenderer {
    pub fn initialise(&mut self, width: u32, height: u32) {

        unsafe {
            // Configure culling
            glEnable(GL_CULL_FACE);
            glCullFace(GL_BACK);
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

        let camera = Camera::new(Vec2::new(width as f32, height as f32), 90.0, 0.1, 100.0);
        self.camera = Option::Some(camera);

        Shader::load_shader(self, "red").expect("failed to load shader");
        let ht2_document = Document::from_file("internal/models/ht2.dae").expect("failed to load ht2.dae");
        let mut ht2_mesh =
            Mesh::new(ht2_document, "ht2-mesh",
                 Option::None,
                 &self.shaders.as_mut().unwrap().get("red").unwrap().clone(), self)
            .expect("failed to create ht2 mesh");
        //ht2_mesh.position = Vec3::new(0.0, 0.25, 4.0);
        self.meshes.as_mut().unwrap().insert("ht2".to_string(), ht2_mesh);
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
            debug!("forward");
            vec += self.camera.as_mut().unwrap().process_keyboard(CameraMovement::Forward, scale);
        }
        if self.keyboard.backward {
            debug!("backward");
            vec += self.camera.as_mut().unwrap().process_keyboard(CameraMovement::Backward, scale);
        }
        if self.keyboard.left {
            debug!("left");
            vec += self.camera.as_mut().unwrap().process_keyboard(CameraMovement::Left, scale);
        }
        if self.keyboard.right {
            debug!("right");
            vec += self.camera.as_mut().unwrap().process_keyboard(CameraMovement::Right, scale);
        }
        self.move_camera(vec);
    }

    pub fn move_camera(&mut self, direction: Vec3) {
        let position = self.camera.as_mut().unwrap().get_position();
        self.camera.as_mut().unwrap().set_position(position + direction);
    }

    pub fn start_rotate_camera(&mut self, mouse_x: f32, mouse_y: f32) {
        self.first_mouse_pos = (mouse_x, mouse_y);
        self.camera_can_move = true;
    }

    pub fn end_rotate_camera(&mut self) {
        self.camera_can_move = false;
    }

    pub fn rotate_camera(&mut self, mouse_x_offset: f32, mouse_y_offset: f32) {
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

    // should be called upon the render action of our GtkGLArea
    pub fn render(&mut self, worldmachine: &mut WorldMachine) {
        // todo! this is a hack
        if !self.initialised {
            self.initialise(1280, 720);
            debug!("initialised renderer");
            worldmachine.initialise();
            debug!("initialised worldmachine");
            self.initialised = true;
        }

        self.process_inputs();

        unsafe {
            // set the clear color to black
            glClearColor(0.1, 0.0, 0.1, 1.0);
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            worldmachine.render(self);

            glFlush();
        }
    }
}