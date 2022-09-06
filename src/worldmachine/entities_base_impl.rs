use std::any::Any;
use std::collections::HashMap;
use std::ops::Deref;
use gfx_maths::*;
use crate::worldmachine::components::*;
use crate::worldmachine::ecs::*;
use super::entities::*;

impl Entity for EntityBase {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_components(&self) -> Vec<&dyn Component> {
        let mut components = Vec::new();
        for (_, component) in self.components.iter() {
            components.push(component.deref());
        }
        components
    }

    fn get_component(&self, component_type: ComponentType) -> Option<&dyn Component> {
        self.components.get(&component_type).map(|component| component.deref())
    }

    fn add_component(&mut self, component: Box<dyn Component>) {
        let component_type = component.get_type();
        self.components.insert(component_type, component);
    }

    fn remove_component(&mut self, component: ComponentType) {
        self.components.remove(&component);
    }

    fn set_component_parameter(&mut self, component_type: ComponentType, parameter_name: &str, value: Box<dyn Any>) {
        if let Some(component) = self.components.get_mut(&component_type) {
            component.set_parameter(parameter_name, value);
        }
    }

    fn get_children(&self) -> Vec<&dyn Entity> {
        let mut children = Vec::new();
        for child in self.children.iter() {
            children.push(child.deref());
        }
        children
    }

    fn add_child(&mut self, mut child: Box<dyn Entity>) {
        self.children.push(child);
    }

    fn remove_child(&mut self, child: Box<dyn Entity>) {
        let child_id = child.get_id();
        self.children.retain(|child| child.get_id() != child_id);
    }
}

impl EntityBase {
pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            id: ENTITY_ID_MANAGER.lock().unwrap().get_id(),
            components: HashMap::new(),
            children: Vec::new(),
            parent: None,
        }
    }
}