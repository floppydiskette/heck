use std::any::Any;
use std::collections::HashMap;
use std::ops::Deref;
use gfx_maths::*;
use crate::worldmachine::components::*;
use crate::worldmachine::components_base_impl::deserialize_component;
use crate::worldmachine::ecs::*;
use crate::worldmachine::helpers;
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

    fn clone(&self) -> Box<dyn Entity> {
        let mut entity = EntityBase{
            name: self.name.clone(),
            id: self.id,
            components: Default::default(),
            children: vec![],
            parent: None
        };
        for (_, component) in self.components.iter() {
            entity.add_component(component.deref().clone());
        }
        for child in self.children.iter() {
            entity.add_child(child.deref().clone());
        }
        Box::new(entity)
    }

    fn to_serialization(&self) -> String {
        let mut serialization = String::new();
        serialization.push_str(&format!("name: {}\n", self.name));
        serialization.push_str(&format!("id: {}\n", self.id));
        serialization.push_str("components:\n");
        for (_, component) in self.components.iter() {
            serialization.push_str(&format!("  - {}\n", component.to_serialization()));
        }
        serialization.push_str("children:\n");
        for child in self.children.iter() {
            serialization.push_str(&format!("  - {}\n", child.to_serialization()));
        }
        serialization
    }

    fn from_serialization(&self, serialization: &str) -> Result<Box<dyn Entity>, String> {
        let mut entity = EntityBase{
            name: String::new(),
            id: 0,
            components: Default::default(),
            children: vec![],
            parent: None
        };
        let mut lines = serialization.lines();
        while let Some(line) = lines.next() {
            let mut split = line.split(':');
            let key = split.next().unwrap().trim();
            let value = split.next().unwrap().trim();
            match key {
                "name" => entity.name = value.to_string(),
                "id" => entity.id = value.parse::<u32>().map_err(|e| {
                    format!("failed to parse id: {}", e)
                })?,
                "components" => {
                    while let Some(component_line) = lines.next() {
                        if component_line.trim().starts_with('-') {
                            //let component_serialization = component_line.trim_start_matches('-').trim();
                            let length_of_start = component_line.find('-').unwrap() + 2; // the 2 is for the space after the dash
                            let component_serialization = &component_line[length_of_start..];
                            debug!("component serialization: {}", component_serialization);
                            let component = deserialize_component(component_serialization)?;
                            entity.add_component(component);
                        } else {
                            break;
                        }
                    }
                },
                "children" => {
                    while let Some(child_line) = lines.next() {
                        if child_line.trim().starts_with('-') {
                            //let child_serialization = child_line.trim_start_matches('-').trim();
                            let length_of_start = child_line.find('-').unwrap() + 2; // the 2 is for the space after the dash
                            let child_serialization = &child_line[length_of_start..];
                            debug!("child serialization: {}", child_serialization);
                            let tmp = EntityBase::new("");
                            let child = tmp.from_serialization(child_serialization)?;
                            entity.add_child(child);
                        } else {
                            break;
                        }
                    }
                },
                _ => return Err(format!("unknown key: {}", key))
            }
        }
        Ok(Box::new(entity))
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