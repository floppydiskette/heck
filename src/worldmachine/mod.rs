use std::ops::Deref;
use std::sync::{Arc, Mutex};
use gfx_maths::{Quaternion, Vec3};
use crate::Cast;
use crate::renderer::H2eckRenderer;
use crate::worldmachine::components::{COMPONENT_TYPE_MESH_RENDERER, COMPONENT_TYPE_TRANSFORM, MeshRenderer};
use crate::worldmachine::ecs::*;
use crate::worldmachine::entities::new_ht2_entity;

pub mod ecs;
pub mod components;
pub mod components_base_impl;
pub mod entities;
pub mod entities_base_impl;

pub struct World {
    pub entities: Vec<Box<dyn Entity>>,
    pub systems: Vec<Box<dyn System>>,
}

pub struct WorldMachine {
    pub world: World,
    pub game_data_path: String,
}

impl Default for WorldMachine {
    fn default() -> Self {
        let world = World {
            entities: Vec::new(),
            systems: Vec::new(),
        };
        Self {
            world: world,
            game_data_path: String::from(""),
        }
    }
}

impl WorldMachine {
    pub fn initialise(&mut self) {
        let mut ht2 = Box::new(new_ht2_entity());
        ht2.set_component_parameter(COMPONENT_TYPE_TRANSFORM.clone(), "position", Box::new(Vec3::new(0.0, 0.25, 4.0)));
        self.world.entities.push(ht2);
    }

    pub fn render(&self, renderer: &mut H2eckRenderer) {
        debug!("rendering world");
        for entity in self.world.entities.iter() {
            if let Some(mesh_renderer) = entity.get_component(COMPONENT_TYPE_MESH_RENDERER.clone()) {
                debug!("rendering entity {}", entity.get_name());
                if let Some(mesh) = mesh_renderer.get_parameter("mesh") {
                    // get the string value of the mesh
                    let mesh_name = mesh.value.downcast::<String>().unwrap();
                    debug!("rendering mesh {}", mesh_name);
                    // if so, render it
                    let shaders = renderer.shaders.clone().unwrap();
                    let meshes = renderer.meshes.clone().unwrap();
                    let shader = shaders.get("red").unwrap();
                    let mut mesh = meshes.get("ht2").unwrap().clone();

                    // if this entity has a transform, apply it
                    if let Some(transform) = entity.get_component(COMPONENT_TYPE_TRANSFORM.clone()) {
                        if let Some(position) = transform.get_parameter("position") {
                            let position = position.value.downcast::<Vec3>().unwrap();
                            mesh.position += *position;
                        }
                        if let Some(rotation) = transform.get_parameter("rotation") {
                            let rotation = rotation.value.downcast::<Quaternion>().unwrap();
                            mesh.rotation = *rotation;
                        }
                        if let Some(scale) = transform.get_parameter("scale") {
                            let scale = scale.value.downcast::<Vec3>().unwrap();
                            mesh.scale += *scale;
                        }
                    }

                    mesh.render(renderer, shader, false);
                }
            }
        }
    }
}