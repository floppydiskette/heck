use gfx_maths::*;
use crate::renderer::H2eckRenderer;
use crate::renderer::mesh::Mesh;
use crate::renderer::shader::Shader;
use crate::worldmachine::ecs::*;

lazy_static! {
    pub static ref COMPONENT_TYPE_TRANSFORM: ComponentType = ComponentType::create_if_not_exists("Transform");
    pub static ref COMPONENT_TYPE_MESH_RENDERER: ComponentType = ComponentType::create_if_not_exists("MeshRenderer");
    pub static ref COMPONENT_TYPE_BRUSH: ComponentType = ComponentType::create_if_not_exists("Brush");
    pub static ref COMPONENT_TYPE_LIGHT: ComponentType = ComponentType::create_if_not_exists("Light");
}

pub struct Transform {
    pub position: Vec3,
    pub rotation: Quaternion,
    pub scale: Vec3,
    // component stuff
    pub(crate) component_type: ComponentType,
}

pub struct MeshRenderer {
    pub mesh: String,
    pub shader: String,
    pub texture: String,
    // component stuff
    pub(crate) component_type: ComponentType,
}

pub struct Brush {
    pub a: Vec3,
    pub b: Vec3,
    pub mesh: String,
    // component stuff
    pub(crate) component_type: ComponentType,
}

pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    // component stuff
    pub(crate) component_type: ComponentType,
}

impl MeshRenderer {
    #[allow(clippy::field_reassign_with_default)]
    pub fn new_from_mesh(mesh: &str) -> Self {
        let mut default = Self::default();
        default.mesh = mesh.to_string();
        default
    }
}

impl Light {
    #[allow(clippy::field_reassign_with_default)]
    pub fn new(position: Vec3, color: Vec3, intensity: f32) -> Self {
        let mut default = Self::default();
        default.position = position;
        default.color = color;
        default.intensity = intensity;
        default
    }
}