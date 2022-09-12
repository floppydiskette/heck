use std::any::Any;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use gfx_maths::{Quaternion, Vec2, Vec3};
use gtk::subclass::prelude::ObjectSubclassIsExt;
use crate::{Cast, renderer};
use crate::h2eck_window::editor::Editor;
use crate::renderer::H2eckRenderer;
use crate::renderer::raycasting::Ray;
use crate::worldmachine::components::{COMPONENT_TYPE_LIGHT, COMPONENT_TYPE_MESH_RENDERER, COMPONENT_TYPE_TRANSFORM, Light, MeshRenderer};
use crate::worldmachine::ecs::*;
use crate::worldmachine::entities::new_ht2_entity;

pub mod ecs;
pub mod components;
pub mod components_base_impl;
pub mod entities;
pub mod entities_base_impl;
pub mod helpers;

pub struct World {
    pub entities: Vec<Box<dyn Entity>>,
    pub systems: Vec<Box<dyn System>>,
}

impl World {
    pub fn serialize(&self) -> String {
        let mut serialized = String::new();
        for entity in &self.entities {
            serialized += "{";
            serialized += entity.to_serialization().as_str();
            serialized += "}";
        }
        serialized
    }

    pub fn deserialize(serialzation: String) -> Result<Self, String> {
        let mut world = World {
            entities: Vec::new(),
            systems: Vec::new(),
        };

        // first, split the serialization into entities
        let mut entities = Vec::new();
        let mut entity = String::new();
        for c in serialzation.chars() {
            if c == '{' {
                entity = String::new();
            } else if c == '}' {
                entities.push(entity.clone());
            } else {
                entity.push(c);
            }
        }

        // for each entity, deserialize it
        for entity in entities {
            let tmp = new_ht2_entity();
            let actual_entity = tmp.from_serialization(entity.as_str())?;
            world.entities.push(actual_entity);
        }

        Ok(world)
    }
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
        let mut ht2 = Box::new(new_ht2_entity());
        ht2.set_component_parameter(COMPONENT_TYPE_TRANSFORM.clone(), "position", Box::new(Vec3::new(0.0, 5.0, 4.0)));
        let light_component = Box::new(Light::new(Vec3::new(0.0, 1.0, 0.0), Vec3::new(1.0, 1.0, 1.0), 1.0));
        ht2.add_component(light_component);
        self.world.entities.push(ht2);
        self.editor = editor;
        {
            let editor = self.editor.lock().unwrap();
            editor.as_ref().unwrap().imp().regen_model_from_world(&mut self.world);
        }

        // as a test, save and load the world
        //self.save_state_to_file("test_world.map");
        self.load_state_from_file("test_world.map");
    }

    pub fn save_state_to_file(&self, file_path: &str) {
        let serialized = self.world.serialize();
        std::fs::write(file_path, serialized).expect("unable to write file");
    }

    pub fn load_state_from_file(&mut self, file_path: &str) {
        let contents = std::fs::read_to_string(file_path).expect("something went wrong reading the file");
        let world = World::deserialize(contents).unwrap();
        self.world = world;
        let editor = self.editor.lock().unwrap();
        editor.as_ref().unwrap().imp().regen_model_from_world(&mut self.world);
    }

    #[allow(clippy::borrowed_box)]
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
                let mut position = *position.value.downcast_ref::<Vec3>().unwrap();
                let color = light.get_parameter("color").unwrap();
                let color = *color.value.downcast_ref::<Vec3>().unwrap();
                let intensity = light.get_parameter("intensity").unwrap();
                let intensity = *intensity.value.downcast_ref::<f32>().unwrap();
                if let Some(transform) = transform_component {
                    let transform = transform.clone();
                    let trans_position = transform.get_parameter("position").unwrap();
                    let trans_position = trans_position.value.downcast_ref::<Vec3>().unwrap();
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
                    let mesh_name = mesh.value.downcast::<String>().unwrap();
                    // if so, render it
                    let shaders = renderer.shaders.clone().unwrap();
                    let meshes = renderer.meshes.clone().unwrap();
                    let mesh = meshes.get(&*mesh_name);
                    if let Some(mesh) = mesh {
                        let mut mesh = *mesh;
                        let shader = mesh_renderer.get_parameter("shader").unwrap();
                        let texture = mesh_renderer.get_parameter("texture").unwrap();
                        let shader_name = shader.value.downcast::<String>().unwrap();
                        let texture_name = texture.value.downcast::<String>().unwrap();
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

                        mesh.render(renderer, shader, Some(texture));
                    }
                }
            }
        }
    }
}