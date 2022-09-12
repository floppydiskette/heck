use std::any::Any;
use std::collections::HashMap;
use gfx_maths::*;
use crate::renderer::mesh::Mesh;
use crate::worldmachine::ecs::{Component, ComponentType, Parameter};
use crate::worldmachine::helpers;
use super::components::*;

fn assert_component_name_and_get_values_from_serialization(name: &str, serialization: String) -> Option<HashMap<String, String>> {
    // a component serialization will be something like this:
    // "component_name:parameter1[values,values],parameter2[single_value]"
    // so we split the string by the colon and check that the first part is the component name
    let mut split = serialization.split(':');
    if let Some(component_name) = split.next() {
        if component_name == name {
            // if the component name is correct, we split the rest of the string by the comma
            // and then split each part by the square brackets
            // the first part of each split will be the parameter name
            // and the second part will be the parameter values

            // split by commas, but not within square brackets
            let mut parameters = Vec::new();
            let mut current_parameter = String::new();
            let mut bracket_count = 0;
            for c in split.next().unwrap().chars() {
                if c == '[' {
                    bracket_count += 1;
                } else if c == ']' {
                    bracket_count -= 1;
                }
                if c == ',' && bracket_count == 0 {
                    parameters.push(current_parameter);
                    current_parameter = String::new();
                } else {
                    current_parameter.push(c);
                }
            }

            // add the last parameter
            parameters.push(current_parameter);

            // split each parameter by the square brackets
            let mut parameter_values = HashMap::new();
            for parameter in parameters {
                let mut split = parameter.split('[');
                if let Some(parameter_name) = split.next() {
                    if let Some(parameter_values_string) = split.next() {
                        // remove the closing square bracket
                        let parameter_values_string = &parameter_values_string[..parameter_values_string.len() - 1];
                        parameter_values.insert(parameter_name.to_string(), parameter_values_string.to_string());
                    }
                }
            }

            return Some(parameter_values);
        }
    }
    None
}

fn serialize_component_name_and_parameters(name: &str, parameters: &HashMap<String, String>) -> String {
    let mut serialization = String::new();
    serialization.push_str(name.to_lowercase().as_str());
    serialization.push(':');
    for (parameter_name, parameter_values) in parameters {
        serialization.push_str(parameter_name);
        serialization.push('[');
        serialization.push_str(parameter_values);
        serialization.push(']');
        serialization.push(',');
    }
    serialization.pop();
    serialization
}

// TODO! REMEMBER TO ADD COMPONENTS HERE OR ELSE THEY WON'T BE ABLE TO BE DESERIALIZED BECAUSE OF MY LAZY IMPLEMENTATION
pub fn deserialize_component(serialization: &str) -> Result<Box<dyn Component>, String> {
    let split = serialization.split(':').collect::<Vec<&str>>();
    let component_type = split[0];
    match component_type.to_lowercase().as_str() {
        "transform" => {
            let tmp = Transform::default();
            tmp.from_serialization(serialization)
        },
        "meshrenderer" => {
            let tmp = crate::worldmachine::components::MeshRenderer::default();
            tmp.from_serialization(serialization)
        },
        "light" => {
            let tmp = crate::worldmachine::components::Light::default();
            tmp.from_serialization(serialization)
        },
        _ => {
            panic!("unknown component type: {}", component_type);
        }
    }
}

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

    fn to_serialization(&self) -> String {
        let position_serialization = helpers::serialize_vec3(&self.position);
        let rotation_serialization = helpers::serialize_quaternion(&self.rotation);
        let scale_serialization = helpers::serialize_vec3(&self.scale);

        let mut parameters = HashMap::new();
        parameters.insert("position".to_string(), position_serialization);
        parameters.insert("rotation".to_string(), rotation_serialization);
        parameters.insert("scale".to_string(), scale_serialization);
        serialize_component_name_and_parameters(&self.component_type.name, &parameters)
    }

    fn from_serialization(&self, serialization: &str) -> Result<Box<dyn Component>, String> {
        if let Some(parameters) = assert_component_name_and_get_values_from_serialization("transform", serialization.to_string()) {
            let mut transform = Transform::default();
            for (parameter_name, parameter_values) in parameters {
                match parameter_name.as_str() {
                    "position" => {
                        if let Ok(position) = helpers::deserialize_vec3(&parameter_values) {
                            transform.position = position;
                        }
                    }
                    "rotation" => {
                        if let Ok(rotation) = helpers::deserialize_quaternion(&parameter_values) {
                            transform.rotation = rotation;
                        }
                    }
                    "scale" => {
                        if let Ok(scale) = helpers::deserialize_vec3(&parameter_values) {
                            transform.scale = scale;
                        }
                    }
                    _ => {}
                }
            }
            Ok(Box::new(transform))
        } else {
            Err("Failed to deserialize transform".to_string())
        }
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

    fn to_serialization(&self) -> String {
        let mut parameters = HashMap::new();
        parameters.insert("mesh".to_string(), self.mesh.clone());
        parameters.insert("shader".to_string(), self.shader.clone());
        parameters.insert("texture".to_string(), self.texture.clone());
        serialize_component_name_and_parameters(&self.component_type.name, &parameters)
    }

    fn from_serialization(&self, serialization: &str) -> Result<Box<dyn Component>, String> {
        if let Some(parameters) = assert_component_name_and_get_values_from_serialization("meshrenderer", serialization.to_string()) {
            let mut mesh_renderer = MeshRenderer::default();
            for (parameter_name, parameter_values) in parameters {
                match parameter_name.as_str() {
                    "mesh" => {
                        mesh_renderer.mesh = parameter_values;
                    }
                    "shader" => {
                        mesh_renderer.shader = parameter_values;
                    }
                    "texture" => {
                        mesh_renderer.texture = parameter_values;
                    }
                    _ => {}
                }
            }
            Ok(Box::new(mesh_renderer))
        } else {
            Err("Failed to deserialize meshrenderer".to_string())
        }
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

    fn to_serialization(&self) -> String {
        let mut parameters = HashMap::new();
        parameters.insert("position".to_string(), helpers::serialize_vec3(&self.position));
        parameters.insert("color".to_string(), helpers::serialize_vec3(&self.color));
        parameters.insert("intensity".to_string(), self.intensity.to_string());
        serialize_component_name_and_parameters(&self.component_type.name, &parameters)
    }

    fn from_serialization(&self, serialization: &str) -> Result<Box<dyn Component>, String> {
        if let Some(parameters) = assert_component_name_and_get_values_from_serialization("light", serialization.to_string()) {
            let mut light = Light::default();
            for (parameter_name, parameter_values) in parameters {
                match parameter_name.as_str() {
                    "position" => {
                        light.position = helpers::deserialize_vec3(&parameter_values)?;
                    }
                    "color" => {
                        light.color = helpers::deserialize_vec3(&parameter_values)?;
                    }
                    "intensity" => {
                        light.intensity = parameter_values.parse::<f32>().unwrap();
                    }
                    _ => {}
                }
            }
            Ok(Box::new(light))
        } else {
            Err("Failed to deserialize light".to_string())
        }
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