use std::collections::HashMap;
use gfx_maths::*;
use crate::worldmachine::components::*;
use crate::worldmachine::ecs::*;

impl Entity {
    pub fn new(name: &str) -> Entity {
        Self {
            name: name.to_string(),
            uid: ENTITY_ID_MANAGER.lock().unwrap().get_id(),
            components: Vec::new(),
            children: Vec::new(),
            parent: None,
        }
    }

    pub fn add_component(&mut self, component: Component) {
        self.components.push(component);
    }
}

pub fn new_ht2_entity() -> Entity {
    let mut entity = Entity::new("ht2");
    entity.add_component(Transform::default());
    entity.add_component(MeshRenderer::default());
    entity
}