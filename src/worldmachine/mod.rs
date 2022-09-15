use std::any::Any;
use std::borrow::{Borrow, BorrowMut};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard};
use gfx_maths::{Quaternion, Vec2, Vec3};
use gtk::subclass::prelude::ObjectSubclassIsExt;
use png::DecodingError::Parameter;
use serde::{Deserialize, Serialize};
use crate::{Cast, renderer};
use crate::h2eck_window::editor::Editor;
use crate::renderer::camera::Camera;
use crate::renderer::H2eckRenderer;
use crate::renderer::raycasting::Ray;
use crate::worldmachine::components::{COMPONENT_TYPE_LIGHT, COMPONENT_TYPE_MESH_RENDERER, COMPONENT_TYPE_TERRAIN, COMPONENT_TYPE_TRANSFORM, Light, MeshRenderer, Terrain, Transform};
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
    eid_manager: u64,
}

#[derive(Deserialize, Serialize)]
pub struct WorldDef {
    pub name: String,
    pub world: World,
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
            eid_manager: 0,
        }
    }
}

pub struct WorldMachine {
    pub world: World,
    pub game_data_path: String,
    pub counter: f32,
    pub editor: Arc<Mutex<Option<Editor>>>,
    pub entities_wanting_to_load_things: Vec<usize>, // index
    lights_changed: bool,
}

impl Default for WorldMachine {
    fn default() -> Self {
        let world = World {
            entities: Vec::new(),
            systems: Vec::new(),
            eid_manager: 0,
        };
        Self {
            world,
            game_data_path: String::from(""),
            counter: 0.0,
            editor: Arc::new(Mutex::new(Option::None)),
            entities_wanting_to_load_things: Vec::new(),
            lights_changed: true,
        }
    }
}

impl WorldMachine {
    pub fn initialise(&mut self, editor: Arc<Mutex<Option<Editor>>>) {
        // todo! get this from settings
        self.game_data_path = String::from("../huskyTech2/base");
        components::register_component_types();

        self.editor = editor;
        self.blank_slate();
    }

    fn regen_editor(&mut self) {
        {
            let editor = self.editor.lock().unwrap();
            if editor.is_none() {
                return;
            }
            editor.as_ref().unwrap().imp().regen_model_from_world(&mut self.world);
        }
    }

    // resets the world to a blank slate
    pub fn blank_slate(&mut self) {
        {
            let mut eid_manager = ENTITY_ID_MANAGER.lock().unwrap();
            eid_manager.borrow_mut().id = 0;
        }
        self.world.entities.clear();
        self.world.systems.clear();
        self.counter = 0.0;
        self.lights_changed = true;
        let mut ht2 = new_ht2_entity();
        ht2.set_component_parameter(COMPONENT_TYPE_TRANSFORM.clone(), "position", ParameterValue::Vec3(Vec3::new(0.0, 0.0, 2.0)));
        let light_component = Light::new(Vec3::new(0.0, 1.0, 0.0), Vec3::new(1.0, 1.0, 1.0), 1.0);
        ht2.add_component(light_component);
        self.world.entities.push(ht2);
        self.regen_editor();
    }

    pub fn load_entity_def(&mut self, name: &str, center_at_camera: Option<Camera>) {
        debug!("{}, {}", name, self.game_data_path);
        let path = format!("{}/entities/{}.edef", self.game_data_path, name);
        let serialization = std::fs::read_to_string(path);
        if serialization.is_err() {
            error!("failed to load entity def: {}", name);
            return;
        }
        let serialization = serialization.unwrap();
        let entity_def: EntityDef = serde_yaml::from_str(&serialization).unwrap();
        let mut entity = Entity::from_entity_def(&entity_def);
        if let Some(camera) = center_at_camera {
            let mut position = camera.get_position();
            position *= -1.0; // camera position is inverted for some reason
            // if the entity has a transform component, set it's position to the raycast position
            if entity.has_component(COMPONENT_TYPE_TRANSFORM.clone()) {
                entity.set_component_parameter(COMPONENT_TYPE_TRANSFORM.clone(), "position", ParameterValue::Vec3(position));
            }
        }
        self.world.entities.push(entity);
        self.entities_wanting_to_load_things.push(self.world.entities.len() - 1);
        self.regen_editor();
    }

    pub fn add_blank_entity(&mut self, name: &str) {
        let entity = Entity::new(name);
        self.world.entities.push(entity);
        self.regen_editor();
    }

    pub fn save_entity_def(&mut self, uid: u64) {
        debug!("{}, {}", uid, self.game_data_path);
        let entity = self.get_entity(uid).unwrap();
        let entity = entity.lock().unwrap();
        let path = format!("{}/entities/{}.edef", self.game_data_path, entity.name);
        let entity_def = entity.to_entity_def();
        let serialization = serde_yaml::to_string(&entity_def).unwrap();
        let res = std::fs::write(path, serialization);
        if res.is_err() {
            error!("failed to save entity def: {}", res.err().unwrap());
        }
    }

    pub fn list_possible_entities(&self) -> Vec<String> {
        let mut entities = Vec::new();
        let paths = std::fs::read_dir(format!("{}/entities", self.game_data_path));
        if paths.is_err() {
            error!("failed to read entities directory!");
            return entities;
        }
        let paths = paths.unwrap();
        for path in paths {
            let path = path.unwrap().path();
            let path = path.to_str().unwrap();
            let path = path.split("/").collect::<Vec<&str>>();
            let path = path[path.len() - 1];
            let path = path.split(".").collect::<Vec<&str>>();
            let path = path[0];
            entities.push(String::from(path));
        }
        entities
    }

    pub fn give_component_to_entity(&mut self, uid: u64, component: Component) {
        let index = self.get_entity_index(uid);
        if index.is_none() {
            error!("failed to give component to entity, entity {} not found", uid);
            return;
        }
        let index = index.unwrap();
        let entity = self.world.entities.get_mut(index).unwrap();
        entity.add_component(component);
        self.regen_editor();
    }

    pub fn remove_component_from_entity(&mut self, uid: u64, component_type: ComponentType) {
        let index = self.get_entity_index(uid).unwrap();
        let entity = self.world.entities.get_mut(index).unwrap();
        entity.remove_component(component_type);
        self.regen_editor();
    }

    pub fn rename_entity(&mut self, uid: u64, new_name: &str) {
        let index = self.get_entity_index(uid).unwrap();
        let entity = self.world.entities.get_mut(index).unwrap();
        entity.name = String::from(new_name);
        self.regen_editor();
    }

    pub fn list_all_component_types(&self) -> Vec<String> {
        let mut component_types = Vec::new();
        let existing_component_type = COMPONENT_TYPES.lock().unwrap().clone();
        for (name, _) in existing_component_type.iter() {
            component_types.push(name.clone());
        }
        component_types
    }

    pub fn new_component_from_name(&self, name: &str) -> Option<Component> {
        let existing_component_type = COMPONENT_TYPES.lock().unwrap().clone();
        let component_type = existing_component_type.get(name);
        component_type?;
        let component_type = component_type.unwrap();
        match component_type.clone() {
            x if x == COMPONENT_TYPE_TRANSFORM.clone() => {
                Some(Transform::default())
            },
            x if x == COMPONENT_TYPE_MESH_RENDERER.clone() => {
                Some(MeshRenderer::default())
            },
            x if x == COMPONENT_TYPE_LIGHT.clone() => {
                Some(Light::default())
            },
            x if x == COMPONENT_TYPE_TERRAIN.clone() => {
                Some(Terrain::default())
            },
            _ => {
                None
            }
        }
    }

    pub fn save_state_to_file(&mut self, file_path: &str) {
        {
            let mut eid_manager = ENTITY_ID_MANAGER.lock().unwrap();
            self.world.eid_manager = eid_manager.borrow().id;
            self.editor.lock().unwrap().as_mut().unwrap().imp().current_world_path.lock().unwrap().replace(String::from(file_path));
        }
        let serialized = serde_yaml::to_string(&self.world).unwrap();
        std::fs::write(file_path, serialized).expect("unable to write file");
    }

    pub fn load_state_from_file(&mut self, file_path: &str) {
        let contents = std::fs::read_to_string(file_path).expect("something went wrong reading the file");
        let world = serde_yaml::from_str(&contents).unwrap();
        self.world = world;

        {
            let mut eid_manager = ENTITY_ID_MANAGER.lock().unwrap();
            eid_manager.borrow_mut().id = self.world.eid_manager;
            self.editor.lock().unwrap().as_mut().unwrap().imp().current_world_path.lock().unwrap().replace(String::from(file_path));
        }
        self.entities_wanting_to_load_things.clear();
        for i in 0..self.world.entities.len() {
            self.entities_wanting_to_load_things.push(i);
        }
        self.regen_editor();
    }

    pub fn compile_map(&mut self, name: &str) {
        // create a directory for the map (if it doesn't exist)
        let map_dir = format!("{}/maps/{}", self.game_data_path, name);
        let res = std::fs::create_dir_all(map_dir.clone());
        if res.is_err() {
            error!("failed to create map directory: {}", res.err().unwrap());
            return;
        }

        let worlddef = WorldDef {
            name: String::from(name),
            world: self.world.clone(),
        };

        let mut serialized = Vec::new();
        let res = worlddef.serialize(&mut rmp_serde::Serializer::new(&mut serialized));
        if res.is_err() {
            error!("failed to serialize worlddef: {}", res.err().unwrap());
            return;
        }
        // write the worlddef to a file
        let res = std::fs::write(format!("{}/worlddef", map_dir), serialized);
        if res.is_err() {
            error!("failed to write worlddef: {}", res.err().unwrap());
            return;
        }
        info!("wrote worlddef to file");
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

    pub fn remove_entity_at_index(&mut self, index: usize) {
        self.world.entities.remove(index);
        self.regen_editor();
    }

    pub fn attempt_to_set_component_property(&mut self, entity_id: u64, component_name: String, property_name: String, value: String) {
        // as component properties are now different, tell the renderer that the lights have changed
        self.lights_changed = true;

        debug!("attempt_to_set_component_property: entity_id: {}, component_name: {}, property_name: {}, value: {}", entity_id, component_name, property_name, value);
        let mut entity_index = self.get_entity_index(entity_id).unwrap();
        self.entities_wanting_to_load_things.push(entity_index);
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
            ParameterValue::String(cv) => {
                let mut value = value.clone();
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &*property_name, ParameterValue::String(value));
            },
            ParameterValue::Float(cv) => {
                let value = value.parse::<f64>().unwrap_or(cv);
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &*property_name, ParameterValue::Float(value));
            },
            ParameterValue::Vec3(cv) => {
                let mut value = value;
                let mut x = 0.0;
                let mut y = 0.0;
                let mut z = 0.0;
                if value.contains(",") {
                    let mut split = value.split(",");
                    x = split.next().unwrap().parse::<f32>().unwrap_or(cv.x);
                    y = split.next().unwrap().parse::<f32>().unwrap_or(cv.y);
                    z = split.next().unwrap().parse::<f32>().unwrap_or(cv.z);
                } else {
                    x = value.parse::<f32>().unwrap_or(cv.x);
                    y = value.parse::<f32>().unwrap_or(cv.y);
                    z = value.parse::<f32>().unwrap_or(cv.z);
                }
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, ParameterValue::Vec3(Vec3::new(x, y, z)));
            },
            ParameterValue::Quaternion(cv) => {
                let mut value = value;
                let mut x = 0.0;
                let mut y = 0.0;
                let mut z = 0.0;
                let mut w = 0.0;
                if value.contains(",") {
                    let mut split = value.split(",");
                    x = split.next().unwrap().parse::<f32>().unwrap_or(cv.x);
                    y = split.next().unwrap().parse::<f32>().unwrap_or(cv.y);
                    z = split.next().unwrap().parse::<f32>().unwrap_or(cv.z);
                    w = split.next().unwrap().parse::<f32>().unwrap_or(cv.w);
                } else {
                    x = value.parse::<f32>().unwrap_or(cv.x);
                    y = value.parse::<f32>().unwrap_or(cv.y);
                    z = value.parse::<f32>().unwrap_or(cv.z);
                    w = value.parse::<f32>().unwrap_or(cv.w);
                }
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, ParameterValue::Quaternion(Quaternion::new(x, y, z, w)));
            },
            ParameterValue::Bool(cv) => {
                let value = value.parse::<bool>().unwrap_or(cv);
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, ParameterValue::Bool(value));
            },
            ParameterValue::UnsignedInt(cv) => {
                let value = value.parse::<u64>().unwrap_or(cv);
                self.world.entities[entity_index].set_component_parameter(component.get_type(), &property_name, ParameterValue::UnsignedInt(value));
            },
            ParameterValue::Int(cv) => {
                let value = value.parse::<i32>().unwrap_or(cv);
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
        for index in self.entities_wanting_to_load_things.clone() {
            let entity = &self.world.entities[index];
            let components = entity.get_components();
            for component in components {
                match component.get_type() {
                    x if x == COMPONENT_TYPE_MESH_RENDERER.clone() => {
                        let mesh = component.get_parameter("mesh").unwrap();
                        let mesh = match &mesh.value {
                            ParameterValue::String(v) => Some(v),
                            _ => {
                                error!("render: mesh is not a string");
                                None
                            }
                        };
                        let mesh = mesh.unwrap();
                        let texture = component.get_parameter("texture").unwrap();
                        let texture = match &texture.value {
                            ParameterValue::String(v) => Some(v),
                            _ => {
                                error!("render: texture is not a string");
                                None
                            }
                        };
                        let texture = texture.unwrap();
                        let res = renderer.load_mesh_if_not_already_loaded(&mesh);
                        if res.is_err() {
                            warn!("render: failed to load mesh: {:?}", res);
                        }
                        let res = renderer.load_texture_if_not_already_loaded(texture);
                        if res.is_err() {
                            warn!("render: failed to load texture: {:?}", res);
                        }
                    }
                    x if x == COMPONENT_TYPE_TERRAIN.clone() => {
                        let name = component.get_parameter("name").unwrap();
                        let name = match &name.value {
                            ParameterValue::String(v) => Some(v),
                            _ => {
                                error!("render: terrain name is not a string");
                                None
                            }
                        };
                        let name = name.unwrap();
                        let res = renderer.load_terrain_if_not_already_loaded(name);
                        if res.is_err() {
                            warn!("render: failed to load terrain: {:?}", res);
                        }
                    }
                    _ => {}
                }
            }
        }
        self.entities_wanting_to_load_things.clear();
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
            if let Some(terrain) = entity.get_component(COMPONENT_TYPE_TERRAIN.clone()) {
                if let Some(name) = terrain.get_parameter("name") {
                    // get the string value of the mesh
                    let name = match name.value {
                        ParameterValue::String(ref s) => s.clone(),
                        _ => {
                            error!("render: terrain name is not a string");
                            continue;
                        }
                    };
                    // if so, render it
                    let terrains = renderer.terrains.clone().unwrap();
                    let terrain = terrains.get(&*name);
                    if let Some(terrain) = terrain {
                        let mut terrain = terrain.clone();
                        if let Some(transform) = entity.get_component(COMPONENT_TYPE_TRANSFORM.clone()) {
                            let position = transform.get_parameter("position").unwrap();
                            let position = match position.value {
                                ParameterValue::Vec3(v) => v,
                                _ => {
                                    error!("render: transform position is not a vec3");
                                    continue;
                                }
                            };
                            let rotation = transform.get_parameter("rotation").unwrap();
                            let rotation = match rotation.value {
                                ParameterValue::Quaternion(v) => v,
                                _ => {
                                    error!("render: transform rotation is not a quaternion");
                                    continue;
                                }
                            };
                            let scale = transform.get_parameter("scale").unwrap();
                            let scale = match scale.value {
                                ParameterValue::Vec3(v) => v,
                                _ => {
                                    error!("render: transform scale is not a vec3");
                                    continue;
                                }
                            };
                            terrain.mesh.position += position;
                            terrain.mesh.rotation = rotation;
                            terrain.mesh.scale += scale;
                        }
                        terrain.render(renderer);
                    }
                }
            }
        }
    }
}