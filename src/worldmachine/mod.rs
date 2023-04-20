use std::any::Any;
use std::borrow::{Borrow, BorrowMut};
use std::collections::{VecDeque};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use fyrox_sound::context::SoundContext;
use gfx_maths::{Quaternion, Vec2, Vec3};
use gl_matrix::common::Quat;
use serde::{Deserialize, Serialize};
use crate::camera::Camera;
use crate::{ht_renderer, renderer};
use crate::animgraph::{AnimGraph, AnimGraphNode};
use crate::audio::AudioBackend;
use crate::common_anim::move_anim::{Features, MoveAnim};
use crate::helpers::{add_quaternion, from_q64, multiply_quaternion, rotate_vector_by_quaternion, to_q64};
use crate::worldmachine::components::{COMPONENT_TYPE_BOX_COLLIDER, COMPONENT_TYPE_JUKEBOX, COMPONENT_TYPE_LIGHT, COMPONENT_TYPE_MESH_RENDERER, COMPONENT_TYPE_PLAYER, COMPONENT_TYPE_TERRAIN, COMPONENT_TYPE_TRANSFORM, COMPONENT_TYPE_TRIGGER, Light, MeshRenderer, Transform};
use crate::worldmachine::ecs::*;
use crate::worldmachine::MapLoadError::FolderNotFound;

pub mod ecs;
pub mod components;
pub mod entities;
pub mod helpers;
pub mod savestates;

pub type EntityId = u64;

#[derive(Deserialize, Serialize)]
pub struct World {
    pub entities: Vec<Entity>,
    pub systems: Vec<System>,
    eid_manager: EntityId,
    pub current_map: String,
}

#[derive(Deserialize, Serialize)]
pub struct WorldDef {
    pub name: String,
    pub world: World,
}

#[derive(Clone, Debug)]
pub enum MapLoadError {
    FolderNotFound(String),
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
            current_map: "".to_string(),
        }
    }
}

pub struct WorldMachine {
    pub world: World,
    pub game_data_path: String,
    pub counter: f32,
    pub entities_wanting_to_load_things: Vec<usize>,
    pub selected_entity: Option<EntityId>,
    // index
    lights_changed: bool,
}

impl Default for WorldMachine {
    fn default() -> Self {
        let world = World {
            entities: Vec::new(),
            systems: Vec::new(),
            eid_manager: 0,
            current_map: "".to_string(),
        };
        Self {
            world,
            game_data_path: String::from(""),
            counter: 0.0,
            entities_wanting_to_load_things: Vec::new(),
            selected_entity: None,
            lights_changed: true,
        }
    }
}

impl WorldMachine {
    pub fn initialise(&mut self) {
        let _ = *components::COMPONENTS_INITIALISED;
        self.game_data_path = String::from("base");

        self.blank_slate();
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

        let mut ht2_entity = Entity::new("ht2");
        let mut ht2_transform = Transform::default();
        ht2_transform.parameters.get_mut("position").unwrap().value = ParameterValue::Vec3(Vec3::new(0.0, 0.0, 5.0));
        let mut ht2_mesh_renderer = MeshRenderer::default();
        ht2_entity.add_component(ht2_transform);
        ht2_entity.add_component(ht2_mesh_renderer);

        self.world.entities.push(ht2_entity);
    }

    pub fn load_map(&mut self, map_name: &str) -> Result<(), MapLoadError> {
        self.blank_slate();
        let map_dir = format!("{}/maps/{}", self.game_data_path, map_name);
        if !std::path::Path::new(&map_dir).exists() {
            return Err(FolderNotFound(map_dir));
        }
        let mut deserializer = rmp_serde::Deserializer::new(std::fs::File::open(format!("{}/worlddef", map_dir)).unwrap());
        let world_def: WorldDef = Deserialize::deserialize(&mut deserializer).unwrap();

        // load entities
        for entity in world_def.world.entities {
            let mut entity_new = Entity::new(entity.name.as_str());
            for component in entity.components {
                let component_type = ComponentType::get(component.get_type().name);
                if component_type.is_none() {
                    panic!("component type not found: {}", component.get_type().name);
                }
                let component_type = component_type.unwrap();
                let mut component = component;
                component.component_type = component_type.clone();

                entity_new.add_component(component);
            }
            self.world.entities.push(entity_new);
        }

        self.world.current_map = map_name.to_string();

        // initialise entities
        self.initialise_entities();

        // load systems
        for system in world_def.world.systems {
            self.world.systems.push(system);
        }

        Ok(())
    }

    /// this should only be called once per map load
    pub fn initialise_entities(&mut self) {
        for entity in &mut self.world.entities {
        }
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_entity(&self, entity_id: EntityId) -> Option<Arc<Mutex<&Entity>>> {
        for entity in self.world.entities.iter() {
            if entity.get_id() == entity_id {
                return Some(Arc::new(Mutex::new(entity)));
            }
        }
        None
    }

    pub fn get_entity_index(&self, entity_id: EntityId) -> Option<usize> {
        for (index, entity) in self.world.entities.iter().enumerate() {
            if entity.get_id() == entity_id {
                return Some(index);
            }
        }
        None
    }

    /*
    pub fn set_entity_position(&mut self, entity_id: EntityId, position: Vec3) {
        let entity_index = self.get_entity_index(entity_id).unwrap();
        let entity = self.world.entities[entity_index].borrow_mut();
        let res = entity.set_component_parameter(COMPONENT_TYPE_TRANSFORM.clone(), "position", ParameterValue::Vec3(position));
        if res.is_none() {
            warn!("attempted to set entity position on an entity that has no transform component");
        }
    }
     */

    pub fn remove_entity_at_index(&mut self, index: usize) {
        self.world.entities.remove(index);
    }

    pub fn send_lights_to_renderer(&mut self) -> Option<Vec<crate::light::Light>> {
        //if !self.lights_changed {
        //    return Option::None;
        //}
        let mut lights = Vec::new();
        for entity in &self.world.entities {
            let components = entity.get_components();
            let mut light_component = None;
            let mut transform_component = None; // if we have a transform component, this will be added to the light's position
            for component in components {
                if component.get_type() == COMPONENT_TYPE_LIGHT.clone() {
                    light_component = Some(component);
                }
                if component.get_type() == COMPONENT_TYPE_TRANSFORM.clone() {
                    transform_component = Some(component);
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
                    position += trans_position;
                }
                lights.push(crate::light::Light {
                    position,
                    color,
                    intensity: intensity as f32,
                });
            }
        }
        self.lights_changed = false;
        Some(lights)
    }

    pub fn render(&mut self, renderer: &mut ht_renderer, shadow_pass: Option<(u8, usize)>) {
        let lights = self.send_lights_to_renderer();
        if lights.is_some() {
            renderer.set_lights(lights.unwrap());
        }
        let mut indices_to_remove = Vec::new();
        for index in self.entities_wanting_to_load_things.clone() {
            let entity = &self.world.entities[index];
            let components = entity.get_components();
            let mut finished_loading = components.len();
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
                        let res = renderer.load_mesh_if_not_already_loaded(mesh);
                        if res.is_err() {
                            warn!("render: failed to load mesh '{}': {:?}", mesh, res);
                        }
                        let mesh_loaded = res.unwrap();
                        let res = renderer.load_texture_if_not_already_loaded(texture);
                        if res.is_err() {
                            warn!("render: failed to load texture '{}': {:?}", texture, res);
                        }
                        let texture_loaded = res.unwrap();
                        if mesh_loaded && texture_loaded {
                            finished_loading -= 1;
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
                        /*let res = renderer.load_terrain_if_not_already_loaded(name);
                        if res.is_err() {
                            warn!("render: failed to load terrain: {:?}", res);
                        }
                         */
                        let terrain_loaded = true;
                        if terrain_loaded {
                            finished_loading -= 1;
                        }
                    }
                    x if x == COMPONENT_TYPE_LIGHT.clone() => {
                        self.lights_changed = true;
                        finished_loading -= 1;
                    }
                    _ => {
                        finished_loading -= 1;
                    }
                }
            }
            if finished_loading == 0 {
                indices_to_remove.push(index);
            }
        }
        self.entities_wanting_to_load_things.retain(|x| !indices_to_remove.contains(x));
        for (i, entity) in self.world.entities.iter_mut().enumerate() {
            if self.entities_wanting_to_load_things.contains(&i) {
                continue;
            }
            if let Some(mesh_renderer) = entity.get_component(COMPONENT_TYPE_MESH_RENDERER.clone()) {
                if let Some(mesh) = mesh_renderer.get_parameter("mesh") {
                    // get the string value of the mesh
                    let mut mesh_name = match mesh.value {
                        ParameterValue::String(ref s) => s.clone(),
                        _ => {
                            error!("render: mesh is not a string");
                            continue;
                        }
                    };
                    // if so, render it
                    let mesh = renderer.meshes.get(&*mesh_name).cloned();
                    if mesh.is_none() {
                        let res = renderer.load_mesh_if_not_already_loaded(&mesh_name);
                        if res.is_err() {
                            error!("render: failed to load mesh '{}': {:?}", mesh_name, res);
                        }
                        continue;
                    }
                    if let Some(mut mesh) = mesh {
                        let texture = mesh_renderer.get_parameter("texture").unwrap();
                        let texture_name = match texture.value {
                            ParameterValue::String(ref s) => s.clone(),
                            _ => {
                                error!("render: texture is not a string");
                                continue;
                            }
                        };
                        let texture = renderer.textures.get(&*texture_name).cloned();
                        if texture.is_none() {
                            let res = renderer.load_texture_if_not_already_loaded(&texture_name);
                            if res.is_err() {
                                error!("render: failed to load texture '{}': {:?}", texture_name, res);
                            }
                            continue;
                        }
                        let texture = texture.unwrap();

                        let old_position = mesh.position;
                        let old_rotation = mesh.rotation;
                        let old_scale = mesh.scale;

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
                                mesh.scale *= scale;
                            }
                        }

                        // add a bit of rotation to the transform to make things more interesting
                        //entity.set_component_parameter(COMPONENT_TYPE_TRANSFORM.clone(), "rotation", Box::new(Quaternion::from_euler_angles_zyx(&Vec3::new(0.0, self.counter, 0.0))));

                        let mut anim_weights = None;
                        if mesh_name == "player" {
                            let move_anim = MoveAnim::from_values(0.0, 0.0);
                            anim_weights = Some(move_anim.weights());
                        }

                        mesh.render(renderer, Some(&texture), anim_weights.clone(), shadow_pass);

                        if let Some(viz) = self.selected_entity {
                            if viz == entity.uid {
                                mesh.render_viz(renderer, Some(&texture), anim_weights, shadow_pass, 1);
                            }
                        }

                        mesh.position = old_position;
                        mesh.rotation = old_rotation;
                        mesh.scale = old_scale;
                        *renderer.meshes.get_mut(&*mesh_name).unwrap() = mesh;
                    } else {
                        // if not, add it to the list of things to load
                        self.entities_wanting_to_load_things.push(i);
                    }
                }
            }
            if let Some(light) = entity.get_component(COMPONENT_TYPE_LIGHT.clone()) {
                if let Some(viz) = self.selected_entity {
                    if viz == entity.uid {
                        let mut mesh = renderer.meshes.get("boxviz").cloned().unwrap();
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

                        mesh.position += match light.get_parameter("position").unwrap().value {
                            ParameterValue::Vec3(v) => v,
                            _ => {
                                error!("render: light position is not a vec3");
                                continue;
                            }
                        };

                        mesh.scale += Vec3::new(0.1, 0.1, 0.1);

                        mesh.render_viz(renderer, None, None, shadow_pass, 2);
                    }
                }
            }

            if let Some(boxcollider) = entity.get_component(COMPONENT_TYPE_BOX_COLLIDER.clone()) {
                if let Some(viz) = self.selected_entity {
                    if viz == entity.uid || {
                        match boxcollider.get_parameter("visualise") {
                            Some(p) => match p.value {
                                ParameterValue::Bool(b) => b,
                                _ => false,
                            },
                            None => false,
                        }
                    } {
                        let mut mesh = renderer.meshes.get("boxviz").cloned().unwrap();

                        mesh.position = match boxcollider.get_parameter("position").unwrap().value {
                            ParameterValue::Vec3(v) => v,
                            _ => {
                                error!("render: box collider position is not a vec3");
                                continue;
                            }
                        };

                        mesh.scale = match boxcollider.get_parameter("scale").unwrap().value {
                            ParameterValue::Vec3(v) => v,
                            _ => {
                                error!("render: box collider scale is not a vec3");
                                continue;
                            }
                        };

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

                        // move the position back by scale to account for how physics works
                        mesh.position.x -= mesh.scale.x / 2.0;
                        mesh.position.y -= mesh.scale.y;
                        mesh.position.z -= -mesh.scale.z / 2.0;

                        mesh.render_viz(renderer, None, None, shadow_pass, 2);
                    }
                }
            }
        }
    }

    pub fn handle_audio(&mut self, renderer: &ht_renderer, audio: &AudioBackend, scontext: &SoundContext) {
        audio.update(renderer.camera.get_position(), -renderer.camera.get_front(), renderer.camera.get_up(), scontext);

        for index in self.entities_wanting_to_load_things.clone() {
            let entity = &self.world.entities[index];
            let components = entity.get_components();
            for component in components {
                match component.get_type() {
                    x if x == COMPONENT_TYPE_JUKEBOX.clone() => {
                        let track = component.get_parameter("track").unwrap();
                        let track = match track.value {
                            ParameterValue::String(ref s) => s.clone(),
                            _ => {
                                error!("audio: jukebox track is not a string");
                                continue;
                            }
                        };
                        // check if the track is already loaded
                        if !audio.is_sound_loaded(&track) {
                            audio.load_sound(&track);
                        }
                    }
                    _ => {}
                }
            }
        }
        // don't clear here because that's done later in rendering


        for (i, entity) in self.world.entities.iter_mut().enumerate() {
            if let Some(jukebox) = entity.get_component(COMPONENT_TYPE_JUKEBOX.clone()) {
                let track = jukebox.get_parameter("track").unwrap();
                let track = match track.value {
                    ParameterValue::String(ref s) => s.clone(),
                    _ => {
                        error!("audio: jukebox track is not a string");
                        continue;
                    }
                };
                let volume = jukebox.get_parameter("volume").unwrap();
                let volume = match volume.value {
                    ParameterValue::Float(v) => v,
                    _ => {
                        error!("audio: jukebox volume is not a float");
                        continue;
                    }
                };
                let playing = jukebox.get_parameter("playing").unwrap();
                let playing = match playing.value {
                    ParameterValue::Bool(ref s) => s.clone(),
                    _ => {
                        error!("audio: jukebox playing is not a string");
                        continue;
                    }
                };
                let uuid = jukebox.get_parameter("uuid").unwrap();
                let uuid = match uuid.value {
                    ParameterValue::String(ref s) => s.clone(),
                    _ => {
                        error!("audio: jukebox uuid is not a string");
                        continue;
                    }
                };

                let position = if let Some(transform) = entity.get_component(COMPONENT_TYPE_TRANSFORM.clone()) {
                    let position = transform.get_parameter("position").unwrap();
                    let position = match position.value {
                        ParameterValue::Vec3(v) => v,
                        _ => {
                            error!("audio: transform position is not a vec3");
                            continue;
                        }
                    };
                    position
                } else {
                    Vec3::new(0.0, 0.0, 0.0)
                };

                if audio.is_sound_loaded(&track) {
                    if playing && !audio.is_sound_playing(&uuid) {
                        audio.play_sound_with_uuid(&uuid, &track, scontext);
                    } else if !playing && audio.is_sound_playing(&uuid) {
                        audio.stop_sound_with_uuid(&uuid, scontext);
                    }
                    if playing {
                        audio.set_sound_position(&uuid, position, scontext);
                    }
                } else {
                    // if not, add it to the list of things to load
                    self.entities_wanting_to_load_things.push(i);
                }
            }
        }
    }
}