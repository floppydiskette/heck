use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use gfx_maths::{Quaternion, Vec2, Vec3};

pub struct Parameter {
    pub name: String,
    pub value: ParameterValue,
}

pub enum ParameterValue {
    Vec3(Vec3),
    Quaternion(Quaternion),
    Vec2(Vec2),
    Float(f32),
    Int(i32),
    Bool(bool),
}

impl Parameter {
    pub(crate) fn new(name: &str, value: ParameterValue) -> Parameter {
        Self {
            name: name.to_string(),
            value,
        }
    }

    pub fn serialize(&self) -> Result<String, String> {
        // turn the value into a string
        // at the moment, this will just match on the type and downcast it
        // to the correct type
        let mut final_str = String::new();
        match self.type_id {
            t if t == TypeId::of::<i32>() => {
                let value = self.value.downcast_ref::<i32>().unwrap();
                final_str = "i32:".to_string() + &value.to_string();
            },
            t if t == TypeId::of::<f32>() => {
                let value = self.value.downcast_ref::<f32>().unwrap();
                final_str = "f32:".to_string() + &value.to_string();
            },
            t if t == TypeId::of::<bool>() => {
                let value = self.value.downcast_ref::<bool>().unwrap();
                final_str = "bool:".to_string() + &value.to_string();
            },
            t if t == TypeId::of::<String>() => {
                let value = self.value.downcast_ref::<String>().unwrap();
                final_str = "string:".to_string() + &value;
            },
            t if t == TypeId::of::<Vec3>() => {
                let value = self.value.downcast_ref::<Vec3>().unwrap();
                final_str = "vec3:".to_string() + &*format!("{},{},{}", value.x, value.y, value.z);
            },
            t if t == TypeId::of::<Vec2>() => {
                let value = self.value.downcast_ref::<Vec2>().unwrap();
                final_str = "vec2:".to_string() + &*format!("{},{}", value.x, value.y);
            },
            t if t == TypeId::of::<Quaternion>() => {
                let value = self.value.downcast_ref::<Quaternion>().unwrap();
                final_str = "quat:".to_string() + &*format!("{},{},{},{}", value.x, value.y, value.z, value.w);
            },
            _ => {
                return Err("unknown type".to_string());
            }
        }

        Ok(final_str)
    }

    pub fn deserialize(name: &str, serialization: &str) -> Result<Self, String> {
        // split the serialization into the type and the value
        let split: Vec<&str> = serialization.split(':').collect();
        if split.len() < 2 {
            return Err("invalid serialization".to_string());
        }

        let value = match split[0] {
            "i32" => {
                let value = split[1].parse::<i32>().unwrap();
                Box::new(value) as Box<dyn Any>
            },
            "f32" => {
                let value = split[1].parse::<f32>().unwrap();
                Box::new(value) as Box<dyn Any>
            },
            "bool" => {
                let value = split[1].parse::<bool>().unwrap();
                Box::new(value) as Box<dyn Any>
            },
            "string" => {
                Box::new(split[0..].join("")) as Box<dyn Any>
            },
            "vec3" => {
                let split: Vec<&str> = split[1].split(',').collect();
                if split.len() != 3 {
                    return Err("invalid serialization".to_string());
                }
                let value = Vec3::new(split[0].parse::<f32>().unwrap(), split[1].parse::<f32>().unwrap(), split[2].parse::<f32>().unwrap());
                Box::new(value) as Box<dyn Any>
            },
            "vec2" => {
                let split: Vec<&str> = split[1].split(',').collect();
                if split.len() != 2 {
                    return Err("invalid serialization".to_string());
                }
                let value = Vec2::new(split[0].parse::<f32>().unwrap(), split[1].parse::<f32>().unwrap());
                Box::new(value) as Box<dyn Any>
            },
            "quat" => {
                let split: Vec<&str> = split[1].split(',').collect();
                if split.len() != 4 {
                    return Err("invalid serialization".to_string());
                }
                let value = Quaternion::new(split[0].parse::<f32>().unwrap(), split[1].parse::<f32>().unwrap(), split[2].parse::<f32>().unwrap(), split[3].parse::<f32>().unwrap());
                Box::new(value) as Box<dyn Any>
            },
            _ => {
                return Err("unknown type".to_string());
            }
        };

        Ok(Self::new(name, value))
    }
}

pub trait Component {
    fn get_id(&self) -> u32;
    fn get_name(&self) -> String;
    fn get_type(&self) -> ComponentType;
    fn get_parameters(&self) -> Vec<Parameter>;
    fn get_parameter(&self, parameter_name: &str) -> Option<Parameter>;
    fn set_parameter(&mut self, parameter_name: &str, value: Box<dyn Any>);
    fn clone(&self) -> Box<dyn Component>;
    fn to_serialization(&self) -> String;
    fn from_serialization(&self, serialization: &str) -> Result<Box<dyn Component>, String>;
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
    fn clone(&self) -> Box<dyn Entity>;
    fn to_serialization(&self) -> String;
    fn from_serialization(&self, serialization: &str) -> Result<Box<dyn Entity>, String>;
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
    fn clone(&self) -> Box<dyn System>;
    fn to_serialization(&self) -> String;
    fn from_serialization(&self, serialization: &str) -> Result<Box<dyn System>, String>;
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