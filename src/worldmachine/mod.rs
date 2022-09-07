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
    pub counter: f32,
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
            counter: 0.0,
        }
    }
}

impl WorldMachine {
    pub fn initialise(&mut self) {
        let mut ht2 = Box::new(new_ht2_entity());
        ht2.set_component_parameter(COMPONENT_TYPE_TRANSFORM.clone(), "position", Box::new(Vec3::new(0.0, 0.25, 4.0)));
        self.world.entities.push(ht2);
    }

    pub fn render(&mut self, renderer: &mut H2eckRenderer) {
        self.counter += 1.0;
        for entity in self.world.entities.iter_mut() {
            if let Some(mesh_renderer) = entity.get_component(COMPONENT_TYPE_MESH_RENDERER.clone()) {
                if let Some(mesh) = mesh_renderer.get_parameter("mesh") {
                    // get the string value of the mesh
                    let mesh_name = mesh.value.downcast::<String>().unwrap();
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
                            // add a bit of rotation to the transform to make things more interesting
                            mesh.rotation = *rotation;
                        }
                        if let Some(scale) = transform.get_parameter("scale") {
                            let scale = scale.value.downcast::<Vec3>().unwrap();
                            mesh.scale += *scale;
                        }
                    }

                    // add a bit of rotation to the transform to make things more interesting
                    //entity.set_component_parameter(COMPONENT_TYPE_TRANSFORM.clone(), "rotation", Box::new(Quaternion::from_euler_angles_zyx(&Vec3::new(0.0, self.counter, 0.0))));

                    mesh.render(renderer, shader, false);
                }
            }
        }
    }
}