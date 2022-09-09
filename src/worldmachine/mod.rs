use std::any::Any;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use gfx_maths::{Quaternion, Vec2, Vec3};
use gtk::subclass::prelude::ObjectSubclassIsExt;
use crate::Cast;
use crate::h2eck_window::editor::Editor;
use crate::renderer::H2eckRenderer;
use crate::renderer::raycasting::Ray;
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

impl Clone for World {
    fn clone(&self) -> Self {
        let mut entities = Vec::new();
        for entity in &self.entities {
            entities.push(entity.deref().clone());
        }
        let mut systems = Vec::new();
        for system in &self.systems {
            systems.push(system.deref().clone());
        }
        World {
            entities,
            systems,
        }
    }
}

pub struct WorldMachine {
    pub world: World,
    pub game_data_path: String,
    pub counter: f32,
    pub editor: Arc<Mutex<Option<Editor>>>
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
            editor: Arc::new(Mutex::new(Option::None))
        }
    }
}

impl WorldMachine {
    pub fn initialise(&mut self, editor: Arc<Mutex<Option<Editor>>>) {
        let mut ht2 = Box::new(new_ht2_entity());
        ht2.set_component_parameter(COMPONENT_TYPE_TRANSFORM.clone(), "position", Box::new(Vec3::new(0.0, 0.25, 4.0)));
        self.world.entities.push(ht2);
        self.editor = editor;
        let editor = self.editor.lock().unwrap();
        editor.as_ref().unwrap().imp().regen_model_from_world(&mut self.world);
    }

    pub fn get_entity(&self, entity_id: u32) -> Option<Arc<Mutex<&Box<dyn Entity>>>> {
        for entity in self.world.entities.iter() {
            if entity.get_id() == entity_id {
                return Some(Arc::new(Mutex::new(entity)));
            }
        }
        None
    }

    pub fn get_entity_index(&self, entity_id: u32) -> Option<usize> {
        for (index, entity) in self.world.entities.iter().enumerate() {
            if entity.get_id() == entity_id {
                return Some(index);
            }
        }
        None
    }

    pub fn attempt_to_set_component_property(&mut self, entity_id: u32, component_name: String, property_name: String, value: String) {
        debug!("attempt_to_set_component_property: entity_id: {}, component_name: {}, property_name: {}, value: {}", entity_id, component_name, property_name, value);
        let mut entity_index = self.get_entity_index(entity_id).unwrap();
        let mut entity = self.world.entities[entity_index].clone();
        let components = entity.get_components();
        let mut component = None;
        for c in components {
            if c.get_name() == component_name {
                component = Some(c);
                break;
            }
        }
        if component.is_none() {
            error!("attempt_to_set_component_property: component not found");
            return;
        }
        let component = component.unwrap();
        let parameter = component.get_parameter(&property_name).unwrap();

        // switch on type
        // if type is string, set string
        // if type is float, convert to float and set float
        // if type is vec3, interpret f32,f32,f32 as x,y,z and set vec3
        // if type is quaternion, interpret f32,f32,f32,f32 as x,y,z,w and set quaternion
        // if type is bool, interpret "true" as true and "false" as false and set bool
        // if type is u32, convert to u32 and set u32
        // if type is i32, convert to i32 and set i32
        let string_type = std::any::TypeId::of::<String>();
        let float_type = std::any::TypeId::of::<f32>();
        let vec3_type = std::any::TypeId::of::<Vec3>();
        let quaternion_type = std::any::TypeId::of::<Quaternion>();
        let bool_type = std::any::TypeId::of::<bool>();
        let u32_type = std::any::TypeId::of::<u32>();
        let i32_type = std::any::TypeId::of::<i32>();
        let type_id = parameter.value.deref().type_id();
        match type_id {
            x if x == string_type => {
                let mut value = value.clone();
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &*property_name, Box::new(value));
            },
            x if x == float_type => {
                let value = value.parse::<f32>().unwrap();
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &*property_name, Box::new(value));
            },
            x if x == vec3_type => {
                let mut value = value;
                let mut x = 0.0;
                let mut y = 0.0;
                let mut z = 0.0;
                if value.contains(",") {
                    let mut split = value.split(",");
                    x = split.next().unwrap().parse::<f32>().unwrap();
                    y = split.next().unwrap().parse::<f32>().unwrap();
                    z = split.next().unwrap().parse::<f32>().unwrap();
                } else {
                    x = value.parse::<f32>().unwrap();
                }
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, Box::new(Vec3::new(x, y, z)));
            },
            x if x == quaternion_type => {
                let mut value = value;
                let mut y = 0.0;
                let mut p = 0.0;
                let mut r = 0.0;
                if value.contains(",") {
                    let mut split = value.split(",");
                    y = split.next().unwrap().parse::<f32>().unwrap();
                    p = split.next().unwrap().parse::<f32>().unwrap();
                    r = split.next().unwrap().parse::<f32>().unwrap();
                } else {
                    y = value.parse::<f32>().unwrap();
                }
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, Box::new(Quaternion::from_euler_angles_zyx(&Vec3::new(y, p, r))));
            },
            x if x == bool_type => {
                let value = value.parse::<bool>().unwrap();
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, Box::new(value));
            },
            x if x == u32_type => {
                let value = value.parse::<u32>().unwrap();
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, Box::new(value));
            },
            x if x == i32_type => {
                let value = value.parse::<i32>().unwrap();
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, Box::new(value));
            },
            _ => {
                error!("attempt_to_set_component_property: unknown type: {:?}", parameter.value);
            }
        }
    }

    pub fn select(&mut self, mouse_x: f32, mouse_y: f32, renderer: &mut H2eckRenderer) {
    }

    pub fn render(&mut self, renderer: &mut H2eckRenderer, selection_buffer: bool) {
        self.counter += 1.0;
        for entity in self.world.entities.iter_mut() {
            if let Some(mesh_renderer) = entity.get_component(COMPONENT_TYPE_MESH_RENDERER.clone()) {
                if let Some(mesh) = mesh_renderer.get_parameter("mesh") {
                    // get the string value of the mesh
                    let mesh_name = mesh.value.downcast::<String>().unwrap();
                    // if so, render it
                    let shaders = renderer.shaders.clone().unwrap();
                    let meshes = renderer.meshes.clone().unwrap();
                    let mesh = meshes.get(&*mesh_name);
                    if let Some(mesh) = mesh {
                        let mut mesh = *mesh;
                        let shader = shaders.get("red").unwrap();

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

                        if selection_buffer {
                            mesh.render(renderer, shader, false, Some(entity.get_id()));
                        } else {
                            mesh.render(renderer, shader, false, None);
                        }
                    }
                }
            }
        }
    }
}