use std::any::Any;
use gfx_maths::*;
use crate::renderer::mesh::Mesh;
use crate::worldmachine::ecs::{Component, ComponentType, Parameter};
use super::components::*;

// transform

impl Component for Transform {
    fn get_id(&self) -> u32 {
        self.component_type.id
    }

    fn get_name(&self) -> String {
        self.component_type.name.clone()
    }

    fn get_type(&self) -> ComponentType {
        self.component_type.clone()
    }

    fn get_parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new("position", Box::new(self.position)),
            Parameter::new("rotation", Box::new(self.rotation)),
            Parameter::new("scale", Box::new(self.scale)),
        ]
    }

    fn get_parameter(&self, parameter_name: &str) -> Option<Parameter> {
        match parameter_name {
            "position" => Some(Parameter::new("position", Box::new(self.position))),
            "rotation" => Some(Parameter::new("rotation", Box::new(self.rotation))),
            "scale" => Some(Parameter::new("scale", Box::new(self.scale))),
            _ => None,
        }
    }

    fn set_parameter(&mut self, parameter_name: &str, parameter: Box<dyn Any>) {
        match parameter_name {
            "position" => {
                if let Ok(value) = parameter.downcast::<Vec3>() {
                    self.position = *value;
                }
            }
            "rotation" => {
                if let Ok(value) = parameter.downcast::<Quaternion>() {
                    self.rotation = *value;
                }
            }
            "scale" => {
                if let Ok(value) = parameter.downcast::<Vec3>() {
                    self.scale = *value;
                }
            }
            _ => {}
        }
    }

    fn clone(&self) -> Box<dyn Component> {
        Box::new(Transform {
            component_type: self.component_type.clone(),
            position: self.position,
            rotation: self.rotation,
            scale: self.scale,
        })
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
            component_type: COMPONENT_TYPE_TRANSFORM.clone(),
        }
    }
}

// meshrenderer

impl Component for MeshRenderer {
    fn get_id(&self) -> u32 {
        self.component_type.id
    }

    fn get_name(&self) -> String {
        self.component_type.name.clone()
    }

    fn get_type(&self) -> ComponentType {
        self.component_type.clone()
    }

    fn get_parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new("mesh", Box::new(self.mesh.clone())),
            Parameter::new("shader", Box::new(self.shader.clone())),
            Parameter::new("texture", Box::new(self.texture.clone())),
        ]
    }

    fn get_parameter(&self, parameter_name: &str) -> Option<Parameter> {
        match parameter_name {
            "mesh" => Some(Parameter::new("mesh", Box::new(self.mesh.clone()))),
            "shader" => Some(Parameter::new("shader", Box::new(self.shader.clone()))),
            "texture" => Some(Parameter::new("texture", Box::new(self.texture.clone()))),
            _ => None,
        }
    }

    fn set_parameter(&mut self, parameter_name: &str, value: Box<dyn Any>) {
        match parameter_name {
            "mesh" => {
                if let Ok(value) = value.downcast::<String>() {
                    self.mesh = *value.clone();
                }
            }
            "shader" => {
                if let Ok(value) = value.downcast::<String>() {
                    self.shader = *value.clone();
                }
            }
            "texture" => {
                if let Ok(value) = value.downcast::<String>() {
                    self.texture = *value.clone();
                }
            }
            _ => {}
        }
    }

    fn clone(&self) -> Box<dyn Component> {
        Box::new(MeshRenderer {
            mesh: self.mesh.clone(),
            shader: self.shader.clone(),
            texture: self.texture.clone(),
            component_type: self.component_type.clone(),
        })
    }
}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self {
            mesh: String::new(),
            shader: "basic".to_string(),
            texture: "default".to_string(),
            component_type: COMPONENT_TYPE_MESH_RENDERER.clone(),
        }
    }
}

// brush
impl Component for Brush {
    fn get_id(&self) -> u32 {
        self.component_type.id
    }

    fn get_name(&self) -> String {
        self.component_type.name.clone()
    }

    fn get_type(&self) -> ComponentType {
        self.component_type.clone()
    }

    fn get_parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new("a", Box::new(self.a)),
            Parameter::new("b", Box::new(self.b)),
            Parameter::new("mesh", Box::new(self.mesh.clone())),
        ]
    }

    fn get_parameter(&self, parameter_name: &str) -> Option<Parameter> {
        match parameter_name {
            "a" => Some(Parameter::new("a", Box::new(self.a))),
            "b" => Some(Parameter::new("b", Box::new(self.b))),
            "mesh" => Some(Parameter::new("mesh", Box::new(self.mesh.clone()))),
            _ => None,
        }
    }

    fn set_parameter(&mut self, parameter_name: &str, value: Box<dyn Any>) {
        match parameter_name {
            "a" => {
                if let Ok(value) = value.downcast::<Vec3>() {
                    self.a = *value;
                }
            }
            "b" => {
                if let Ok(value) = value.downcast::<Vec3>() {
                    self.b = *value;
                }
            }
            "mesh" => {
                if let Ok(value) = value.downcast::<String>() {
                    self.mesh = *value.clone();
                }
            }
            _ => {}
        }
    }

    fn clone(&self) -> Box<dyn Component> {
        Box::new(Brush {
            a: self.a,
            b: self.b,
            mesh: self.mesh.clone(),
            component_type: self.component_type.clone(),
        })
    }
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 1.0, 1.0),
            mesh: String::new(),
            component_type: COMPONENT_TYPE_BRUSH.clone(),
        }
    }
}

// light
impl Component for Light {
    fn get_id(&self) -> u32 {
        self.component_type.id
    }

    fn get_name(&self) -> String {
        self.component_type.name.clone()
    }

    fn get_type(&self) -> ComponentType {
        self.component_type.clone()
    }

    fn get_parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new("position", Box::new(self.position)),
            Parameter::new("color", Box::new(self.color)),
            Parameter::new("intensity", Box::new(self.intensity)),
        ]
    }

    fn get_parameter(&self, parameter_name: &str) -> Option<Parameter> {
        match parameter_name {
            "position" => Some(Parameter::new("position", Box::new(self.position))),
            "color" => Some(Parameter::new("color", Box::new(self.color))),
            "intensity" => Some(Parameter::new("intensity", Box::new(self.intensity))),
            _ => None,
        }
    }

    fn set_parameter(&mut self, parameter_name: &str, value: Box<dyn Any>) {
        match parameter_name {
            "position" => {
                if let Ok(value) = value.downcast::<Vec3>() {
                    self.position = *value;
                }
            }
            "color" => {
                if let Ok(value) = value.downcast::<Vec3>() {
                    self.color = *value;
                }
            }
            "intensity" => {
                if let Ok(value) = value.downcast::<f32>() {
                    self.intensity = *value;
                }
            }
            _ => {}
        }
    }

    fn clone(&self) -> Box<dyn Component> {
        Box::new(Light {
            position: self.position,
            color: self.color,
            intensity: self.intensity,
            component_type: self.component_type.clone(),
        })
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
            component_type: COMPONENT_TYPE_LIGHT.clone(),
        }
    }
}