use gfx_maths::*;
use crate::worldmachine::components::COMPONENT_TYPE_LIGHT;
use crate::worldmachine::ecs::Component;

pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

impl Light {
    pub fn from_component(component: Box<&dyn Component>) -> Option<Light> {
        if component.get_type() == COMPONENT_TYPE_LIGHT.clone() {
            let position = component.get_parameter("position").unwrap();
            let position = position.value.downcast_ref::<Vec3>().unwrap();
            let color = component.get_parameter("color").unwrap();
            let color = color.value.downcast_ref::<Vec3>().unwrap();
            let intensity = component.get_parameter("intensity").unwrap();
            let intensity = intensity.value.downcast_ref::<f32>().unwrap();
            Some(Light {
                position: *position,
                color: *color,
                intensity: *intensity,
            })
        } else {
            None
        }
    }
}