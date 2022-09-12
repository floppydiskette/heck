use std::any::Any;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use gfx_maths::{Quaternion, Vec2, Vec3};
use gtk::subclass::prelude::ObjectSubclassIsExt;
use png::DecodingError::Parameter;
use serde::{Deserialize, Serialize};
use crate::{Cast, renderer};
use crate::h2eck_window::editor::Editor;
use crate::renderer::H2eckRenderer;
use crate::renderer::raycasting::Ray;
use crate::worldmachine::components::{COMPONENT_TYPE_LIGHT, COMPONENT_TYPE_MESH_RENDERER, COMPONENT_TYPE_TRANSFORM, Light, MeshRenderer};
use crate::worldmachine::ecs::*;
use crate::worldmachine::entities::new_ht2_entity;

pub mod ecs;
pub mod components;
pub mod entities;
pub mod helpers;

#[derive(Deserialize, Serialize)]
pub struct World {
    pub entities: Vec<Entity>,
    pub systems: Vec<System>,
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
    pub editor: Arc<Mutex<Option<Editor>>>,
    lights_changed: bool,
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
            editor: Arc::new(Mutex::new(Option::None)),
            lights_changed: true
        }
    }
}

impl WorldMachine {
    pub fn initialise(&mut self, editor: Arc<Mutex<Option<Editor>>>) {
        let mut ht2 = new_ht2_entity();
        ht2.set_component_parameter(COMPONENT_TYPE_TRANSFORM.clone(), "position", ParameterValue::Vec3(Vec3::new(0.0, 5.0, 4.0)));
        let light_component = Light::new(Vec3::new(0.0, 1.0, 0.0), Vec3::new(1.0, 1.0, 1.0), 1.0);
        ht2.add_component(light_component);
        self.world.entities.push(ht2);
        self.editor = editor;
        {
            let editor = self.editor.lock().unwrap();
            editor.as_ref().unwrap().imp().regen_model_from_world(&mut self.world);
        }
    }

    pub fn save_state_to_file(&self, file_path: &str) {
        let serialized = serde_yaml::to_string(&self.world).unwrap();
        std::fs::write(file_path, serialized).expect("unable to write file");
    }

    pub fn load_state_from_file(&mut self, file_path: &str) {
        let contents = std::fs::read_to_string(file_path).expect("something went wrong reading the file");
        let world = serde_yaml::from_str(&contents).unwrap();
        self.world = world;
        let editor = self.editor.lock().unwrap();
        editor.as_ref().unwrap().imp().regen_model_from_world(&mut self.world);
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_entity(&self, entity_id: u64) -> Option<Arc<Mutex<&Entity>>> {
        for entity in self.world.entities.iter() {
            if entity.get_id() == entity_id {
                return Some(Arc::new(Mutex::new(entity)));
            }
        }
        None
    }

    pub fn get_entity_index(&self, entity_id: u64) -> Option<usize> {
        for (index, entity) in self.world.entities.iter().enumerate() {
            if entity.get_id() == entity_id {
                return Some(index);
            }
        }
        None
    }

    pub fn attempt_to_set_component_property(&mut self, entity_id: u64, component_name: String, property_name: String, value: String) {
        // as component properties are now different, tell the renderer that the lights have changed
        self.lights_changed = true;

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
        match parameter.value.clone() {
            ParameterValue::String(_) => {
                let mut value = value.clone();
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &*property_name, ParameterValue::String(value));
            },
            ParameterValue::Float(_) => {
                let value = value.parse::<f64>().unwrap();
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &*property_name, ParameterValue::Float(value));
            },
            ParameterValue::Vec3(_) => {
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
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, ParameterValue::Vec3(Vec3::new(x, y, z)));
            },
            ParameterValue::Quaternion(_) => {
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
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, ParameterValue::Quaternion(Quaternion::from_euler_angles_zyx(&Vec3::new(y, p, r))));
            },
            ParameterValue::Bool(_) => {
                let value = value.parse::<bool>().unwrap();
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, ParameterValue::Bool(value));
            },
            ParameterValue::UnsignedInt(_) => {
                let value = value.parse::<u64>().unwrap();
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, ParameterValue::UnsignedInt(value));
            },
            ParameterValue::Int(_) => {
                let value = value.parse::<i32>().unwrap();
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, ParameterValue::Int(value));
            },
            _ => {
                error!("attempt_to_set_component_property: unknown type: {:?}", parameter.value);
            }
        }
    }

    pub fn select(&mut self, mouse_x: f32, mouse_y: f32, renderer: &mut H2eckRenderer) {
    }

    pub fn send_lights_to_renderer(&self) -> Option<Vec<renderer::light::Light>> {
        if !self.lights_changed {
            return Option::None;
        }
        let mut lights = Vec::new();
        for entity in &self.world.entities {
            let components = entity.get_components();
            let mut light_component = Option::None;
            let mut transform_component = Option::None; // if we have a transform component, this will be added to the light's position
            for component in components {
                if component.get_type() == COMPONENT_TYPE_LIGHT.clone() {
                    light_component = Option::Some(component);
                }
                if component.get_type() == COMPONENT_TYPE_TRANSFORM.clone() {
                    transform_component = Option::Some(component);
                }
            }
            if let Some(light) = light_component {
                let mut light = light.clone();
                let position = light.get_parameter("position").unwrap();
                let mut position = match position.value {
                    ParameterValue::Vec3(v) => v,
                    _ => {
                        error!("send_lights_to_renderer: light position is not a vec3");
                        Vec3::new(0.0, 0.0, 0.0)
                    }
                };
                let color = light.get_parameter("colour").unwrap();
                let color = match color.value {
                    ParameterValue::Vec3(v) => v,
                    _ => {
                        error!("send_lights_to_renderer: light color is not a vec3");
                        Vec3::new(0.0, 0.0, 0.0)
                    }
                };
                let intensity = light.get_parameter("intensity").unwrap();
                let intensity = match intensity.value {
                    ParameterValue::Float(v) => v,
                    _ => {
                        error!("send_lights_to_renderer: light intensity is not a float");
                        0.0
                    }
                };
                if let Some(transform) = transform_component {
                    let transform = transform.clone();
                    let trans_position = transform.get_parameter("position").unwrap();
                    let trans_position = match trans_position.value {
                        ParameterValue::Vec3(v) => v,
                        _ => {
                            error!("send_lights_to_renderer: transform position is not a vec3");
                            Vec3::new(0.0, 0.0, 0.0)
                        }
                    };
                    position = position + trans_position;
                }
                lights.push(renderer::light::Light{
                    position,
                    color,
                    intensity,
                });
            }
        }
        Some(lights)
    }

    pub fn render(&mut self, renderer: &mut H2eckRenderer) {
        self.counter += 1.0;
        for entity in self.world.entities.iter_mut() {
            if let Some(mesh_renderer) = entity.get_component(COMPONENT_TYPE_MESH_RENDERER.clone()) {
                if let Some(mesh) = mesh_renderer.get_parameter("mesh") {
                    // get the string value of the mesh
                    let mesh_name = match mesh.value {
                        ParameterValue::String(ref s) => s.clone(),
                        _ => {
                            error!("render: mesh is not a string");
                            continue;
                        }
                    };
                    // if so, render it
                    let shaders = renderer.shaders.clone().unwrap();
                    let meshes = renderer.meshes.clone().unwrap();
                    let mesh = meshes.get(&*mesh_name);
                    if let Some(mesh) = mesh {
                        let mut mesh = *mesh;
                        let shader = mesh_renderer.get_parameter("shader").unwrap();
                        let texture = mesh_renderer.get_parameter("texture").unwrap();
                        let shader_name = match shader.value {
                            ParameterValue::String(ref s) => s.clone(),
                            _ => {
                                error!("render: shader is not a string");
                                continue;
                            }
                        };
                        let texture_name = match texture.value {
                            ParameterValue::String(ref s) => s.clone(),
                            _ => {
                                error!("render: texture is not a string");
                                continue;
                            }
                        };
                        let shaders = renderer.shaders.clone().unwrap();
                        let textures = renderer.textures.clone().unwrap();
                        let shader = shaders.get(&*shader_name);
                        let texture = textures.get(&*texture_name);
                        if shader.is_none() || texture.is_none() {
                            error!("shader or texture not found: {:?} {:?}", shader_name, texture_name);
                            continue;
                        }
                        let shader = shader.unwrap();
                        let texture = texture.unwrap();

                        // if this entity has a transform, apply it
                        if let Some(transform) = entity.get_component(COMPONENT_TYPE_TRANSFORM.clone()) {
                            if let Some(position) = transform.get_parameter("position") {
                                let position = match position.value {
                                    ParameterValue::Vec3(v) => v,
                                    _ => {
                                        error!("render: transform position is not a vec3");
                                        continue;
                                    }
                                };
                                mesh.position += position;
                            }
                            if let Some(rotation) = transform.get_parameter("rotation") {
                                let rotation = match rotation.value {
                                    ParameterValue::Quaternion(v) => v,
                                    _ => {
                                        error!("render: transform rotation is not a quaternion");
                                        continue;
                                    }
                                };
                                // add a bit of rotation to the transform to make things more interesting
                                mesh.rotation = rotation;
                            }
                            if let Some(scale) = transform.get_parameter("scale") {
                                let scale = match scale.value {
                                    ParameterValue::Vec3(v) => v,
                                    _ => {
                                        error!("render: transform scale is not a vec3");
                                        continue;
                                    }
                                };
                                mesh.scale += scale;
                            }
                        }

                        // add a bit of rotation to the transform to make things more interesting
                        //entity.set_component_parameter(COMPONENT_TYPE_TRANSFORM.clone(), "rotation", Box::new(Quaternion::from_euler_angles_zyx(&Vec3::new(0.0, self.counter, 0.0))));

                        mesh.render(renderer, shader, Some(texture));
                    }
                }
            }
        }
    }
}