use gfx_maths::*;
use crate::renderer::mesh::Mesh;
use crate::worldmachine::ecs::*;

lazy_static! {
    pub static ref COMPONENT_TYPE_TRANSFORM: ComponentType = ComponentType::create_if_not_exists("Transform");
    pub static ref COMPONENT_TYPE_MESH_RENDERER: ComponentType = ComponentType::create_if_not_exists("MeshRenderer");
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