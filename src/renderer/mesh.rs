use std::ffi::CString;
use std::ptr::null;
use gfx_maths::*;
use libsex::bindings::*;
use crate::renderer::{H2eckRenderer, helpers, MAX_LIGHTS};
use crate::renderer::shader::Shader;
use crate::renderer::texture::Texture;

#[derive(Clone, Copy, Debug)]
pub struct Mesh {
    pub position: Vec3,
    pub rotation: Quaternion,
    pub scale: Vec3,
    pub vao: GLuint,
    pub vbo: GLuint,
    pub ebo: GLuint,
    pub num_vertices: usize,
    pub num_indices: usize,
    pub uvbo: GLuint,

    pub top_left: Vec3,
    pub bottom_right: Vec3,

    //pub texture: Option<Texture>,
}

#[derive(Debug)]
pub enum MeshComponent {
    Mesh,
    Tris,
    VerticesMap,
    Vertices,
    SourceMap,
    Source,
    UvSource,
    SourceArray,
    UvSourceArray,
    Indices,
}

#[derive(Debug)]
pub enum MeshError {
    FunctionNotImplemented,
    MeshNotFound,
    MeshNameNotFound,
    MeshComponentNotFound(MeshComponent),
    UnsupportedArrayType,
}

impl Mesh {
    pub fn new(path: &str, mesh_name: &str, shader: &Shader, renderer: &mut H2eckRenderer) -> Result<Self, MeshError> {
        // load from gltf
        let (document, buffers, images) = gltf::import(path).map_err(|_| MeshError::MeshNotFound)?;

        // get the mesh
        debug!("all meshes: {:?}", document.meshes().map(|m| m.name()).collect::<Vec<_>>());
        let mesh = document.meshes().find(|m| m.name() == Some(mesh_name)).ok_or(MeshError::MeshNameNotFound)?;

        // for each primitive in the mesh
        let mut vertices_array = Vec::new();
        let mut indices_array = Vec::new();
        let mut uvs_array = Vec::new();
        let mut normals_array = Vec::new();
        for primitive in mesh.primitives() {
            // get the vertex positions
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let positions = reader.read_positions().ok_or(MeshError::MeshComponentNotFound(MeshComponent::Vertices))?;
            let positions = positions.collect::<Vec<_>>();

            // get the indices
            let indices = reader.read_indices().ok_or(MeshError::MeshComponentNotFound(MeshComponent::Indices))?;
            let indices = indices.into_u32().collect::<Vec<_>>();

            // get the texture coordinates
            let tex_coords = reader.read_tex_coords(0).ok_or(MeshError::MeshComponentNotFound(MeshComponent::UvSource))?;
            let tex_coords = tex_coords.into_f32();
            let tex_coords = tex_coords.collect::<Vec<_>>();

            // get the normals
            let normals = reader.read_normals().ok_or(MeshError::MeshComponentNotFound(MeshComponent::SourceMap))?;
            let normals = normals.collect::<Vec<_>>();

            // add the vertices (with each grouping of three f32s as three separate f32s)
            vertices_array.extend(positions.iter().flat_map(|v| vec![v[0], v[1], v[2]]));

            // add the indices
            indices_array.extend_from_slice(&indices);

            // add the uvs (with each grouping of two f32s as two separate f32s)
            uvs_array.extend(tex_coords.iter().flat_map(|v| vec![v[0], v[1]]));

            // add the normals (with each grouping of three f32s as three separate f32s)
            normals_array.extend(normals.iter().flat_map(|v| vec![v[0], v[1], v[2]]));
        }

        // get the u32 data from the mesh
        let mut vbo = 0 as GLuint;
        let mut vao = 0 as GLuint;
        let mut ebo = 0 as GLuint;
        let mut uvbo= 0 as GLuint;

        unsafe {
            // set the shader program
            if renderer.current_shader != Some(shader.name.clone()) {
                unsafe {
                    glUseProgram(shader.program);
                    renderer.current_shader = Some(shader.name.clone());
                }
            }

            glGenVertexArrays(1, &mut vao);
            glBindVertexArray(vao);
            glGenBuffers(1, &mut vbo);
            glBindBuffer(GL_ARRAY_BUFFER, vbo);
            glBufferData(GL_ARRAY_BUFFER, (vertices_array.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr, vertices_array.as_ptr() as *const GLvoid, GL_STATIC_DRAW);
            // vertex positions for vertex shader
            let pos = glGetAttribLocation(shader.program, CString::new("in_pos").unwrap().as_ptr());
            glVertexAttribPointer(pos as GLuint, 3, GL_FLOAT, GL_FALSE as GLboolean, 0, null());
            glEnableVertexAttribArray(0);

            // uvs
            glGenBuffers(1, &mut uvbo);
            glBindBuffer(GL_ARRAY_BUFFER, uvbo);
            glBufferData(GL_ARRAY_BUFFER, (uvs_array.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr, uvs_array.as_ptr() as *const GLvoid, GL_STATIC_DRAW);
            // vertex uvs for fragment shader
            let uv = glGetAttribLocation(shader.program, CString::new("in_uv").unwrap().as_ptr());
            glVertexAttribPointer(uv as GLuint, 2, GL_FLOAT, GL_TRUE as GLboolean, 0, null());
            glEnableVertexAttribArray(1);

            // normals
            let mut normalbo = 0 as GLuint;
            glGenBuffers(1, &mut normalbo);
            glBindBuffer(GL_ARRAY_BUFFER, normalbo);
            glBufferData(GL_ARRAY_BUFFER, (normals_array.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr, normals_array.as_ptr() as *const GLvoid, GL_STATIC_DRAW);
            // vertex normals for fragment shader
            let normal = glGetAttribLocation(shader.program, CString::new("in_normal").unwrap().as_ptr());
            glVertexAttribPointer(normal as GLuint, 3, GL_FLOAT, GL_TRUE as GLboolean, 0, null());
            glEnableVertexAttribArray(2);


            // now the indices
            glGenBuffers(1, &mut ebo);
            glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
            glBufferData(GL_ELEMENT_ARRAY_BUFFER, (indices_array.len() * std::mem::size_of::<GLuint>()) as GLsizeiptr, indices_array.as_ptr() as *const GLvoid, GL_STATIC_DRAW);
        }


        unsafe {
            let mut error = glGetError();
            while error != GL_NO_ERROR {
                error!("OpenGL error while initialising mesh: {}", error);
                error = glGetError();
            }
        }

        // calculate the bounding box
        let mut min = Vec3::new(0.0, 0.0, 0.0);
        let mut max = Vec3::new(0.0, 0.0, 0.0);



        Ok(Mesh {
            position: Default::default(),
            rotation: Default::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
            vao,
            vbo,
            ebo,
            num_vertices: vertices_array.len(),
            num_indices: indices_array.len(),
            uvbo,
            top_left: min,
            bottom_right: max,
        })
    }

    pub fn render(&self, renderer: &mut H2eckRenderer, shader: &Shader, texture: Option<&Texture>) {
        // load the shader
        if renderer.current_shader != Some(shader.name.clone()) {
            unsafe {
                glUseProgram(shader.program);
                renderer.current_shader = Some(shader.name.clone());
            }
        }
        unsafe {

            glEnableVertexAttribArray(0);
            glBindVertexArray(self.vao);
            if let Some(texture) = texture {
                // send the material struct to the shader
                let material = texture.material;
                let material_diffuse = glGetUniformLocation(shader.program, CString::new("u_material.diffuse").unwrap().as_ptr());
                let material_roughness = glGetUniformLocation(shader.program, CString::new("u_material.roughness").unwrap().as_ptr());
                let material_metallic = glGetUniformLocation(shader.program, CString::new("u_material.metallic").unwrap().as_ptr());
                let material_normal = glGetUniformLocation(shader.program, CString::new("u_material.normal").unwrap().as_ptr());

                // load textures
                glActiveTexture(GL_TEXTURE0);
                glBindTexture(GL_TEXTURE_2D, material.diffuse_texture);
                glUniform1i(material_diffuse, 0);
                glActiveTexture(GL_TEXTURE1);
                glBindTexture(GL_TEXTURE_2D, material.roughness_texture);
                glUniform1i(material_roughness, 1);
                glActiveTexture(GL_TEXTURE2);
                glBindTexture(GL_TEXTURE_2D, material.metallic_texture);
                glUniform1i(material_metallic, 2);
                glActiveTexture(GL_TEXTURE3);
                glBindTexture(GL_TEXTURE_2D, material.normal_texture);
                glUniform1i(material_normal, 3);

            }

            // send the lights to the shader
            let light_count = renderer.lights.len();
            let light_count = if light_count > MAX_LIGHTS { MAX_LIGHTS } else { light_count };
            let light_count_loc = glGetUniformLocation(shader.program, CString::new("u_light_count").unwrap().as_ptr());
            glUniform1i(light_count_loc, light_count as i32);
            for (i, light) in renderer.lights.iter().enumerate() {
                if i >= MAX_LIGHTS { break; }
                let light_pos = glGetUniformLocation(shader.program, CString::new(format!("u_lights[{}].position", i)).unwrap().as_ptr());
                let light_color = glGetUniformLocation(shader.program, CString::new(format!("u_lights[{}].colour", i)).unwrap().as_ptr());
                let light_intensity = glGetUniformLocation(shader.program, CString::new(format!("u_lights[{}].intensity", i)).unwrap().as_ptr());

                glUniform3f(light_pos, light.position.x, light.position.y, light.position.z);
                glUniform3f(light_color, light.color.x, light.color.y, light.color.z);
                glUniform1f(light_intensity, light.intensity as f32);
            }

            // transformation time!
            let camera_projection = renderer.camera.as_mut().unwrap().get_projection();
            let camera_view = renderer.camera.as_mut().unwrap().get_view();

            // calculate the model matrix
            let model_matrix = calculate_model_matrix(self.position, self.rotation, self.scale);

            // calculate the mvp matrix
            let mvp = camera_projection * camera_view * model_matrix;

            // send the mvp matrix to the shader
            let mvp_loc = glGetUniformLocation(shader.program, CString::new("u_mvp").unwrap().as_ptr());
            glUniformMatrix4fv(mvp_loc, 1, GL_FALSE as GLboolean, mvp.as_ptr());

            // send the model matrix to the shader
            let model_loc = glGetUniformLocation(shader.program, CString::new("u_model").unwrap().as_ptr());
            glUniformMatrix4fv(model_loc, 1, GL_FALSE as GLboolean, model_matrix.as_ptr());

            // send the camera position to the shader
            let camera_pos_loc = glGetUniformLocation(shader.program, CString::new("u_camera_pos").unwrap().as_ptr());
            glUniform3f(camera_pos_loc,
                        renderer.camera.as_mut().unwrap().get_position().x,
                        renderer.camera.as_mut().unwrap().get_position().y,
                        renderer.camera.as_mut().unwrap().get_position().z);

            glDrawElements(GL_TRIANGLES, self.num_indices as GLsizei, GL_UNSIGNED_INT, null());

            // print opengl errors
            let mut error = glGetError();
            while error != GL_NO_ERROR {
                error!("OpenGL error while rendering: {}", error);
                error = glGetError();
            }
        }
    }
}

fn calculate_model_matrix(position: Vec3, rotation: Quaternion, scale: Vec3) -> Mat4 {
    let mut model_matrix = Mat4::identity();
    model_matrix = model_matrix * Mat4::translate(position);
    model_matrix = model_matrix * Mat4::rotate(rotation);
    model_matrix = model_matrix * Mat4::scale(scale);
    model_matrix
}