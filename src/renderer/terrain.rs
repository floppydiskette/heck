use std::ffi::CString;
use std::ptr::null;
use gfx_maths::{Mat4, Quaternion, Vec3};
use libsex::bindings::*;
use crate::renderer::{H2eckRenderer, MAX_LIGHTS};
use crate::renderer::mesh::Mesh;
use crate::renderer::shader::Shader;
use crate::renderer::texture::Texture;

#[derive(Clone)]
pub struct Terrain {
    pub mesh: Mesh,
    pub shader: Shader,
    pub mixmap: Texture,
    pub textures: [Texture; 4],
}

impl Terrain {
    pub fn new_from_name(name: &str, renderer: &mut H2eckRenderer) -> Result<Self, String> {
        // get shader
        let shader = renderer.shaders.as_mut().unwrap().get("terrain").ok_or("Could not find shader")?.clone();
        // load mesh
        let mut mesh = Mesh::new(format!("{}/terrains/{}.glb", renderer.data_dir, name).as_str(), "terrain", &shader, renderer).map_err(|e| format!("failed to load terrain: {:?}", e))?;
        mesh.scale = Vec3::new(20.0, 20.0, 20.0);
        // load textures
        let mixmap = Texture::new_from_name(
            format!("{}/terrains/{}_mixmap", renderer.data_dir, name),
            renderer, true)
            .map_err(|e| format!("failed to load terrain: {:?}", e))?;
        let textures = [
            *renderer.textures.clone().unwrap().get("grass1").ok_or("Could not find texture")?,
            *renderer.textures.clone().unwrap().get("dirt1").ok_or("Could not find texture")?,
            *renderer.textures.clone().unwrap().get("rock1").ok_or("Could not find texture")?,
            *renderer.textures.clone().unwrap().get("sand1").ok_or("Could not find texture")?,
        ];
        Ok(Self {
            mesh,
            shader,
            mixmap,
            textures,
        })
    }

    pub fn render(&self, renderer: &mut H2eckRenderer) {
        // load the shader
        if renderer.current_shader != Some(self.shader.name.clone()) {
            unsafe {
                glUseProgram(self.shader.program);
                renderer.current_shader = Some(self.shader.name.clone());
            }
        }
        unsafe {
            glEnableVertexAttribArray(0);
            glBindVertexArray(self.mesh.vao);
            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, self.mixmap.diffuse_texture);
            glActiveTexture(GL_TEXTURE1);
            glBindTexture(GL_TEXTURE_2D, self.textures[0].diffuse_texture);
            glActiveTexture(GL_TEXTURE2);
            glBindTexture(GL_TEXTURE_2D, self.textures[1].diffuse_texture);
            glActiveTexture(GL_TEXTURE3);
            glBindTexture(GL_TEXTURE_2D, self.textures[2].diffuse_texture);
            glActiveTexture(GL_TEXTURE4);
            glBindTexture(GL_TEXTURE_2D, self.textures[3].diffuse_texture);
            glUniform1i(glGetUniformLocation(self.shader.program, CString::new("mixmap").unwrap().as_ptr()), 0);
            glUniform1i(glGetUniformLocation(self.shader.program, CString::new("tex0").unwrap().as_ptr()), 1);
            glUniform1i(glGetUniformLocation(self.shader.program, CString::new("tex1").unwrap().as_ptr()), 2);
            glUniform1i(glGetUniformLocation(self.shader.program, CString::new("tex2").unwrap().as_ptr()), 3);
            glUniform1i(glGetUniformLocation(self.shader.program, CString::new("tex3").unwrap().as_ptr()), 4);

            // send the lights to the shader
            let light_count = renderer.lights.len();
            let light_count = if light_count > MAX_LIGHTS { MAX_LIGHTS } else { light_count };
            let light_count_loc = glGetUniformLocation(self.shader.program, CString::new("u_light_count").unwrap().as_ptr());
            glUniform1i(light_count_loc, light_count as i32);
            for (i, light) in renderer.lights.iter().enumerate() {
                if i >= MAX_LIGHTS { break; }
                let light_pos = glGetUniformLocation(self.shader.program, CString::new(format!("u_lights[{}].position", i)).unwrap().as_ptr());
                let light_color = glGetUniformLocation(self.shader.program, CString::new(format!("u_lights[{}].colour", i)).unwrap().as_ptr());
                let light_intensity = glGetUniformLocation(self.shader.program, CString::new(format!("u_lights[{}].intensity", i)).unwrap().as_ptr());

                glUniform3f(light_pos, light.position.x, light.position.y, light.position.z);
                glUniform3f(light_color, light.color.x, light.color.y, light.color.z);
                glUniform1f(light_intensity, light.intensity);
            }


            // transformation time!
            let camera_projection = renderer.camera.as_mut().unwrap().get_projection();
            let camera_view = renderer.camera.as_mut().unwrap().get_view();

            // calculate the model matrix
            let model_matrix = calculate_model_matrix(Vec3::new(0.0, 0.0, 0.0), Quaternion::identity(), Vec3::new(1.0, 1.0, 1.0));

            // calculate the mvp matrix
            let mvp = camera_projection * camera_view * model_matrix;

            // send the mvp matrix to the shader
            let mvp_loc = glGetUniformLocation(self.shader.program, CString::new("u_mvp").unwrap().as_ptr());
            glUniformMatrix4fv(mvp_loc, 1, GL_FALSE as GLboolean, mvp.as_ptr());


            // send the model matrix to the shader
            let model_loc = glGetUniformLocation(self.shader.program, CString::new("u_model").unwrap().as_ptr());
            glUniformMatrix4fv(model_loc, 1, GL_FALSE as GLboolean, model_matrix.as_ptr());

            // send the camera position to the shader
            let camera_pos_loc = glGetUniformLocation(self.shader.program, CString::new("u_camera_pos").unwrap().as_ptr());
            glUniform3f(camera_pos_loc,
                        renderer.camera.as_mut().unwrap().get_position().x,
                        renderer.camera.as_mut().unwrap().get_position().y,
                        renderer.camera.as_mut().unwrap().get_position().z);

            glDrawElements(GL_TRIANGLES, self.mesh.num_indices as GLsizei, GL_UNSIGNED_INT, null());
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