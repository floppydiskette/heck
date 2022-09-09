use std::ffi::CString;
use std::ptr::null;
use dae_parser::{ArrayElement, Document, Geometry, Source, Vertices};
use gfx_maths::*;
use libsex::bindings::*;
use crate::renderer::H2eckRenderer;
use crate::renderer::shader::Shader;
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
    pub fn new(doc: Document, mesh_name: &str, texture_location: Option<&str>, shader: &Shader, renderer: &mut H2eckRenderer) -> Result<Self, MeshError> {
        // load from dae
        let geom = doc.local_map::<Geometry>().map_err(|_| MeshError::MeshNotFound)?.get_str(&*mesh_name).ok_or(MeshError::MeshNotFound)?;
        let mesh = geom.element.as_mesh().ok_or(MeshError::MeshComponentNotFound(MeshComponent::Mesh))?;
        let tris = mesh.elements[0].as_triangles().ok_or(MeshError::MeshComponentNotFound(MeshComponent::Tris))?;
        let vertices_map = doc.local_map::<Vertices>().map_err(|_| MeshError::MeshComponentNotFound(MeshComponent::VerticesMap))?;
        let vertices = vertices_map.get_raw(&tris.inputs[0].source).ok_or(MeshError::MeshComponentNotFound(MeshComponent::Vertices))?;
        let source_map = doc.local_map::<Source>().map_err(|_| MeshError::MeshComponentNotFound(MeshComponent::SourceMap))?;
        let source = source_map.get_raw(&vertices.inputs[0].source).ok_or(MeshError::MeshComponentNotFound(MeshComponent::Source))?;
        let uv_source = source_map.get_raw(&tris.inputs[2].source).ok_or(MeshError::MeshComponentNotFound(MeshComponent::UvSource))?;

        let array = source.array.clone().ok_or(MeshError::MeshComponentNotFound(MeshComponent::SourceArray))?;
        let uv_array = uv_source.array.clone().ok_or(MeshError::MeshComponentNotFound(MeshComponent::UvSourceArray))?;

        // get the u32 data from the mesh
        let mut vbo = 0 as GLuint;
        let mut vao = 0 as GLuint;
        let mut ebo = 0 as GLuint;
        let mut uvbo= 0 as GLuint;
        let mut indices = tris.data.clone().prim.expect("no indices?");
        // 9 accounts for the x3 needed to convert to triangles, and the x3 needed to skip the normals and tex coords
        let num_indices = tris.count * 9;

        // todo: this only counts for triangulated collada meshes made in blender, we cannot assume that everything else will act like this

        // indices for vertex positions are offset by the normal and texcoord indices
        // we need to skip the normal and texcoord indices and fill a new array with the vertex positions
        let mut new_indices = Vec::with_capacity(num_indices);
        let mut new_uv_indices = Vec::with_capacity(num_indices);
        // skip the normal (offset 1) and texcoord (offset 2) indices
        let mut i = 0;
        while i < num_indices {
            new_indices.push(indices[i] as u32);
            new_uv_indices.push(indices[i + 2] as u32);
            i += 3;
        }


        let indices = new_indices;
        let num_indices = indices.len();
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
            // assuming that the world hasn't imploded, the array should be either a float array or an int array
            // the array is currently an ArrayElement enum, we need to get the inner value
            let mut size;
            if let ArrayElement::Float(a) = array {
                debug!("len: {}", a.val.len());
                debug!("type: float");
                size = a.val.len() * std::mem::size_of::<f32>();
                glBufferData(GL_ARRAY_BUFFER, size as GLsizeiptr, a.val.as_ptr() as *const GLvoid, GL_STATIC_DRAW);
            } else if let ArrayElement::Int(a) = array {
                debug!("len: {}", a.val.len());
                debug!("type: int");
                size = a.val.len() * std::mem::size_of::<i32>();
            } else {
                panic!("unsupported array type");
            }
            // vertex positions for vertex shader
            let pos = glGetAttribLocation(shader.program, CString::new("in_pos").unwrap().as_ptr());
            glVertexAttribPointer(pos as GLuint, 3, GL_FLOAT, GL_FALSE as GLboolean, 0, null());
            glEnableVertexAttribArray(0);

            //// uvs
            //glGenBuffers(1, &mut uvbo);
            //glBindBuffer(GL_ARRAY_BUFFER, uvbo);
            //if let ArrayElement::Float(a) = uv_array {
            //    debug!("len: {}", a.val.len());
            //    debug!("type: float");
            //    size = a.val.len() * std::mem::size_of::<f32>();
            //    glBufferData(GL_ARRAY_BUFFER, size as GLsizeiptr, a.val.as_ptr() as *const GLvoid, GL_STATIC_DRAW);
            //} else {
            //    panic!("unsupported array type for uvs");
            //}
            //// vertex uvs for fragment shader
            //let uv = glGetAttribLocation(shader.program, CString::new("in_uv").unwrap().as_ptr());
            //glVertexAttribPointer(uv as GLuint, 2, GL_FLOAT, GL_FALSE as GLboolean, 0, null());
            //glEnableVertexAttribArray(1);


            // now the indices
            glGenBuffers(1, &mut ebo);
            glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
            size = num_indices * std::mem::size_of::<i32>();
            glBufferData(GL_ELEMENT_ARRAY_BUFFER, size as GLsizeiptr, indices.as_ptr() as *const GLvoid, GL_STATIC_DRAW);
        }

        let array = source.array.clone().expect("NO ARRAY?");


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
        if let ArrayElement::Float(a) = array.clone() {
            for i in 0..a.val.len() {
                if i % 3 == 0 {
                    if a.val[i] < min.x {
                        min.x = a.val[i];
                    }
                    if a.val[i] > max.x {
                        max.x = a.val[i];
                    }
                } else if i % 3 == 1 {
                    if a.val[i] < min.y {
                        min.y = a.val[i];
                    }
                    if a.val[i] > max.y {
                        max.y = a.val[i];
                    }
                } else if i % 3 == 2 {
                    if a.val[i] < min.z {
                        min.z = a.val[i];
                    }
                    if a.val[i] > max.z {
                        max.z = a.val[i];
                    }
                }
            }
        }



        if let ArrayElement::Float(array) = array {
            let num_vertices = array.val.len();
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
        } else if let ArrayElement::Int(array) = array {
            let num_vertices = array.val.len();
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
        } else {
            Err(MeshError::UnsupportedArrayType)
        }
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

            //// uvs
            //glGenBuffers(1, &mut uvbo);
            //glBindBuffer(GL_ARRAY_BUFFER, uvbo);
            //if let ArrayElement::Float(a) = uv_array {
            //    debug!("len: {}", a.val.len());
            //    debug!("type: float");
            //    size = a.val.len() * std::mem::size_of::<f32>();
            //    glBufferData(GL_ARRAY_BUFFER, size as GLsizeiptr, a.val.as_ptr() as *const GLvoid, GL_STATIC_DRAW);
            //} else {
            //    panic!("unsupported array type for uvs");
            //}
            //// vertex uvs for fragment shader
            //let uv = glGetAttribLocation(shader.program, CString::new("in_uv").unwrap().as_ptr());
            //glVertexAttribPointer(uv as GLuint, 2, GL_FLOAT, GL_FALSE as GLboolean, 0, null());
            //glEnableVertexAttribArray(1);


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

    pub fn render(&self, renderer: &mut H2eckRenderer, shader: &Shader, pass_texture: bool, selection_buffer: Option<u32>) {
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
            if pass_texture {
                //glActiveTexture(GL_TEXTURE0);
                //glBindTexture(GL_TEXTURE_2D, self.texture.unwrap().diffuse_texture);
                //glUniform1i(glGetUniformLocation(renderer.shaders.as_mut().unwrap()[shader_index].program, CString::new("u_texture").unwrap().as_ptr()), 0);
                // DON'T PRINT OPEN GL ERRORS HERE! BIGGEST MISTAKE OF MY LIFE
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

        // secondly, render the selection buffer
        if let Some(entity_id) = selection_buffer {
            unsafe {
                glBindFramebuffer(GL_FRAMEBUFFER, renderer.selection_framebuffer as GLuint);

                glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

                // get the "picking" shader
                let picking_shader = renderer.shaders.as_mut().unwrap().get("picking").unwrap();

                // load the shader
                if renderer.current_shader != Some(picking_shader.name.clone()) {
                    glUseProgram(picking_shader.program);
                    renderer.current_shader = Some(picking_shader.name.clone());
                }

                // load the model and transform it again
                glEnableVertexAttribArray(0);
                glBindVertexArray(self.vao);
                if pass_texture {
                    //glActiveTexture(GL_TEXTURE0);
                    //glBindTexture(GL_TEXTURE_2D, self.texture.unwrap().diffuse_texture);
                    //glUniform1i(glGetUniformLocation(renderer.shaders.as_mut().unwrap()[shader_index].program, CString::new("u_texture").unwrap().as_ptr()), 0);
                    // DON'T PRINT OPEN GL ERRORS HERE! BIGGEST MISTAKE OF MY LIFE
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
                glUniformMatrix4fv(mvp_loc, 1, GL_TRUE as GLboolean, mvp.as_ptr());
                // send the entity id to the shader
                let entity_id_loc = glGetUniformLocation(shader.program, CString::new("u_entity_id").unwrap().as_ptr());
                glUniform1ui(entity_id_loc, entity_id as GLuint);

                glDrawElements(GL_TRIANGLES, self.num_indices as GLsizei, GL_UNSIGNED_INT, null());
                glDisableVertexAttribArray(0);

                // set uniform back to default
                let entity_id_loc = glGetUniformLocation(shader.program, CString::new("u_entity_id").unwrap().as_ptr());
                glUniform1ui(entity_id_loc, 0);

                // print opengl errors
                let mut error = glGetError();
                while error != GL_NO_ERROR {
                    error!("OpenGL error while rendering (selection buffer): {}", error);
                    error = glGetError();
                }

                // reset the framebuffer
                glBindFramebuffer(GL_FRAMEBUFFER, 0);
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