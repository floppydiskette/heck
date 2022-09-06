use std::any::Any;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

pub struct Parameter {
    pub name: String,
    pub value: Box<dyn Any>
}

impl Parameter {
    pub(crate) fn new(name: &str, value: Box<dyn Any>) -> Self {
        Self {
            name: name.to_string(),
            value
        }
    }
}

pub trait Component {
    fn get_id(&self) -> u32;
    fn get_name(&self) -> String;
    fn get_type(&self) -> ComponentType;
    fn get_parameters(&self) -> Vec<Parameter>;
    fn get_parameter(&self, parameter_name: &str) -> Option<Parameter>;
    fn set_parameter(&mut self, parameter_name: &str, value: Box<dyn Any>);
}

pub trait Entity {
    fn get_id(&self) -> u32;
    fn get_name(&self) -> String;
    fn get_components(&self) -> Vec<&dyn Component>;
    fn get_component(&self, component_type: ComponentType) -> Option<&dyn Component>;
    fn add_component(&mut self, component: Box<dyn Component>);
    fn remove_component(&mut self, component_type: ComponentType);
    fn set_component_parameter(&mut self, component_type: ComponentType, parameter_name: &str, value: Box<dyn Any>);
    fn get_children(&self) -> Vec<&dyn Entity>;
    fn add_child(&mut self, child: Box<dyn Entity>);
    fn remove_child(&mut self, child: Box<dyn Entity>);
}

pub trait System {
    fn get_id(&self) -> u32;
    fn get_name(&self) -> String;
    fn get_type(&self) -> SystemType;
    fn get_entities(&self) -> Vec<Box<dyn Entity>>;
    fn get_entity(&self, entity_id: u32) -> Option<Box<dyn Entity>>;
    fn add_entity(&mut self, entity: Box<dyn Entity>);
    fn remove_entity(&mut self, entity_id: u32);
    fn update(&mut self);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentType {
    pub id: u32,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct SystemType {
    pub id: u32,
    pub name: String,
}

impl ComponentType {
    pub fn create(name: &str) {
        let id = COMPONENT_ID_MANAGER.lock().unwrap().get_id();
        let mut hashmap = COMPONENT_TYPES.lock().unwrap();
        let component_type = Self {
            id,
            name: name.to_string(),
        };
        hashmap.insert(name.to_string(), component_type);
    }

    pub fn create_if_not_exists(name: &str) -> Self {
        let mut hashmap = COMPONENT_TYPES.lock().unwrap();
        hashmap.entry(name.to_string()).or_insert_with(|| {
            let id = COMPONENT_ID_MANAGER.lock().unwrap().get_id();
            let component_type = Self {
                id,
                name: name.to_string(),
            };
            component_type.clone()
        }).deref().clone()
    }

    pub fn get(name: String) -> Option<Self> {
        COMPONENT_TYPES.lock().unwrap().get(&*name).cloned()
    }
}

impl SystemType {
    pub fn create(hashmap: &mut HashMap<String, Self>, name: String) {
        let id = SYSTEM_ID_MANAGER.lock().unwrap().get_id();
        let system_type = Self {
            id,
            name: name.clone(),
        };
        hashmap.insert(name, system_type);
    }

    pub fn get(name: String) -> Option<Self> {
        SYSTEM_TYPES.lock().unwrap().get(&*name).cloned()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ComponentIDManager {
    pub id: u32,
}

#[derive(Clone, Debug, Default)]
pub struct SystemIDManager {
    pub id: u32,
}

#[derive(Clone, Debug, Default)]
pub struct EntityIDManager {
    pub id: u32,
}

impl ComponentIDManager {
    pub fn get_id(&mut self) -> u32 {
        self.id += 1;
        self.id
    }
}

impl SystemIDManager {
    pub fn get_id(&mut self) -> u32 {
        self.id += 1;
        self.id
    }
}

impl EntityIDManager {
    pub fn get_id(&mut self) -> u32 {
        self.id += 1;
        self.id
    }
}

lazy_static! {
    pub static ref COMPONENT_ID_MANAGER: Mutex<ComponentIDManager> = Mutex::new(ComponentIDManager::default());
    pub static ref COMPONENT_TYPES: Mutex<HashMap<String, ComponentType>> = {
        let mut m = HashMap::new();
        Mutex::new(m)
    };
    pub static ref SYSTEM_ID_MANAGER: Mutex<SystemIDManager> = Mutex::new(SystemIDManager::default());
    pub static ref SYSTEM_TYPES: Mutex<HashMap<String, SystemType>> = {
        let mut m = HashMap::new();
        Mutex::new(m)
    };
    pub static ref ENTITY_ID_MANAGER: Mutex<EntityIDManager> = Mutex::new(EntityIDManager::default());
}