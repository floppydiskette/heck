use std::ffi::CString;
use std::ptr::null;
use gfx_maths::*;
use libsex::bindings::*;
use crate::renderer::{H2eckRenderer, helpers};
use crate::renderer::shader::Shader;
use crate::renderer::texture::Texture;
use crate::worldmachine::components::Brush;

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
    MeshComponentNotFound(MeshComponent),
    UnsupportedArrayType,
}

impl Mesh {
    pub fn new(path: &str, mesh_name: &str, shader: &Shader, renderer: &mut H2eckRenderer) -> Result<Self, MeshError> {
        // load from gltf
        let (document, buffers, images) = gltf::import(path).map_err(|_| MeshError::MeshNotFound)?;

        // get the mesh
        let mesh = document.meshes().find(|m| m.name() == Some(mesh_name)).ok_or(MeshError::MeshNotFound)?;

        // for each primitive in the mesh
        let mut vertices_array = Vec::new();
        let mut indices_array = Vec::new();
        let mut uvs_array = Vec::new();
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

            // add the vertices (with each grouping of three f32s as three separate f32s)
            vertices_array.extend(positions.iter().flat_map(|v| vec![v[0], v[1], v[2]]));

            // add the indices
            indices_array.extend_from_slice(&indices);

            // add the uvs (with each grouping of two f32s as two separate f32s)
            uvs_array.extend(tex_coords.iter().flat_map(|v| vec![v[0], v[1]]));
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

    pub fn new_brush_mesh(brush: &Brush, shader: &Shader, renderer: &mut H2eckRenderer) -> Result<Self, MeshError> {
        let point_a = brush.a;
        let point_b = brush.b;
        // generate a cube with the brush's dimensions
        let vertices: Vec<f32> = vec![
            // front
            point_a.x, point_a.y, point_b.z,
            point_b.x, point_a.y, point_b.z,
            point_b.x, point_b.y, point_b.z,
            point_a.x, point_b.y, point_b.z,
            // back
            point_a.x, point_a.y, point_a.z,
            point_b.x, point_a.y, point_a.z,
            point_b.x, point_b.y, point_a.z,
            point_a.x, point_b.y, point_a.z,
        ];
        let indices = vec![
            // front
            0, 1, 2,
            2, 3, 0,
            // right
            1, 5, 6,
            6, 2, 1,
            // back
            7, 6, 5,
            5, 4, 7,
            // left
            4, 0, 3,
            3, 7, 4,
            // top
            4, 5, 1,
            1, 0, 4,
            // bottom
            3, 2, 6,
            6, 7, 3,
        ];

        let uvs = vec![
            // front
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            // back
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
        ];

        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        let mut uvbo = 0;
        let num_indices = indices.len();
        let num_vertices = vertices.len();

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
            glBufferData(GL_ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr, vertices.as_ptr() as *const GLvoid, GL_STATIC_DRAW);
            // vertex positions for vertex shader
            let pos = glGetAttribLocation(shader.program, CString::new("in_pos").unwrap().as_ptr());
            glVertexAttribPointer(pos as GLuint, 3, GL_FLOAT, GL_FALSE as GLboolean, 0, null());
            glEnableVertexAttribArray(0);

            // uvs
            glGenBuffers(1, &mut uvbo);
            glBindBuffer(GL_ARRAY_BUFFER, uvbo);
            glBufferData(GL_ARRAY_BUFFER, (uvs.len() * std::mem::size_of::<f32>()) as GLsizeiptr, uvs.as_ptr() as *const GLvoid, GL_STATIC_DRAW);
            // vertex uvs for fragment shader
            let uv = glGetAttribLocation(shader.program, CString::new("in_uv").unwrap().as_ptr());
            glVertexAttribPointer(uv as GLuint, 2, GL_FLOAT, GL_FALSE as GLboolean, 0, null());
            glEnableVertexAttribArray(1);


            // now the indices
            glGenBuffers(1, &mut ebo);
            glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
            glBufferData(GL_ELEMENT_ARRAY_BUFFER, (indices.len() * std::mem::size_of::<i32>()) as GLsizeiptr, indices.as_ptr() as *const GLvoid, GL_STATIC_DRAW);
        }

        unsafe {
            let mut error = glGetError();
            while error != GL_NO_ERROR {
                error!("OpenGL error while initialising mesh: {}", error);
                error = glGetError();
            }
        }

        // calculate the bounding box
        let min = point_a;
        let max = point_b;

        Ok(Mesh {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
            vbo,
            vao,
            ebo,
            uvbo,
            top_left: min,
            bottom_right: max,
            num_vertices,
            num_indices,
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
                glActiveTexture(GL_TEXTURE0);
                glBindTexture(GL_TEXTURE_2D, texture.diffuse_texture);
                glUniform1i(glGetUniformLocation(shader.program, CString::new("u_texture").unwrap().as_ptr()), 0);
                //DON'T PRINT OPEN GL ERRORS HERE! BIGGEST MISTAKE OF MY LIFE
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

            glDrawElements(GL_TRIANGLES, self.num_indices as GLsizei, GL_UNSIGNED_INT, null());
            glDisableVertexAttribArray(0);

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