use std::collections::HashMap;
use gfx_maths::*;
use crate::worldmachine::components::*;
use crate::worldmachine::ecs::*;

pub struct EntityBase {
    pub name: String,
    pub id: u32,
    pub components: HashMap<ComponentType, Box<dyn Component>>,
    pub children: Vec<Box<dyn Entity>>,
    pub parent: Option<Box<dyn Entity>>,
}

pub fn new_ht2_entity() -> EntityBase {
    let mut entity = EntityBase::new("ht2");
    entity.add_component(Box::new(Transform::default()));
    entity.add_component(Box::new(MeshRenderer::new_from_mesh("ht2")));
    entity
}