mod mesh;
mod helpers;
mod types;
mod shader;
mod camera;

use std::collections::HashMap;
use dae_parser::Document;
use gfx_maths::{Vec2, Vec3};
use libsex::bindings::*;
use crate::renderer::camera::Camera;
use crate::renderer::mesh::Mesh;
use crate::renderer::shader::Shader;
use crate::renderer::types::*;

pub struct H2eckRenderer {
    pub state: H2eckState,
    pub camera: Option<Camera>,
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

        let camera = Camera::new(Vec2::new(width as f32, height as f32), 45.0, 0.1, 100.0);
        self.camera = Option::Some(camera);

        Shader::load_shader(self, "red").expect("failed to load shader");
        let ht2_document = Document::from_file("internal/models/ht2.dae").expect("failed to load ht2.dae");
        let mut ht2_mesh =
            Mesh::new(ht2_document, "ht2-mesh",
                 Option::None,
                 &self.shaders.as_mut().unwrap().get("red").unwrap().clone(), self)
            .expect("failed to create ht2 mesh");
        ht2_mesh.position = Vec3::new(0.0, 0.25, 4.0);
        self.meshes.as_mut().unwrap().insert("ht2".to_string(), ht2_mesh);
    }

    // should be called upon the render action of our GtkGLArea
    pub fn render(&mut self) {
        // todo! this is a hack
        if !self.initialised {
            self.initialise(800, 600);
            self.initialised = true;
        }

        unsafe {
            // set the clear color to black
            glClearColor(1.0, 0.0, 1.0, 1.0);
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            debug!("meshes: {:?}", self.meshes);

            if self.meshes.is_some() && self.meshes.clone().unwrap().contains_key("ht2") {
                debug!("rendering ht2");
                let shaders = self.shaders.clone().unwrap();
                let meshes = self.meshes.clone().unwrap();
                let shader = shaders.get("red").unwrap();
                let mesh = meshes.get("ht2").unwrap();
                mesh.render(self, shader, false);
            }

            glFlush();
        }
    }
}