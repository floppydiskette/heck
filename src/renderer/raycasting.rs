use std::ops::Deref;
use gfx_maths::*;
use crate::renderer::camera::Camera;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec4,
    pub max_distance: f32,
}

pub fn convert_screen_coords_to_device_coords(screen_coords: Vec2, window_size: Vec2) -> Vec2 {
    let x = 2.0 * screen_coords.x / window_size.x - 1.0;
    let y = 1.0 - (2.0 * screen_coords.y) / window_size.y;
    Vec2::new(x, y)
}

pub fn inverse(mat: Mat4) -> Mat4 {
    use gl_matrix::mat4;
    // convert to gl_matrix mat4
    let mut gl_mat = mat4::create();
    gl_mat[0] = mat.get(0, 0);
    gl_mat[1] = mat.get(0, 1);
    gl_mat[2] = mat.get(0, 2);
    gl_mat[3] = mat.get(0, 3);
    gl_mat[4] = mat.get(1, 0);
    gl_mat[5] = mat.get(1, 1);
    gl_mat[6] = mat.get(1, 2);
    gl_mat[7] = mat.get(1, 3);
    gl_mat[8] = mat.get(2, 0);
    gl_mat[9] = mat.get(2, 1);
    gl_mat[10] = mat.get(2, 2);
    gl_mat[11] = mat.get(2, 3);
    gl_mat[12] = mat.get(3, 0);
    gl_mat[13] = mat.get(3, 1);
    gl_mat[14] = mat.get(3, 2);
    gl_mat[15] = mat.get(3, 3);

    let mut out_mat = mat4::create();
    mat4::invert(&mut out_mat, &gl_mat);

    // convert back to gfx_maths mat4
    Mat4::from([
        out_mat[0], out_mat[1], out_mat[2], out_mat[3],
        out_mat[4], out_mat[5], out_mat[6], out_mat[7],
        out_mat[8], out_mat[9], out_mat[10], out_mat[11],
        out_mat[12], out_mat[13], out_mat[14], out_mat[15],
    ])
}

pub fn xyz(vec: Vec4) -> Vec3 {
    Vec3::new(vec.x, vec.y, vec.z)
}

pub fn distance_between_two_points(a: Vec3, b: Vec3) -> f32 {
    let x = a.x - b.x;
    let y = a.y - b.y;
    let z = a.z - b.z;
    (x * x + y * y + z * z).sqrt()
}

pub fn length(vec: Vec3) -> f32 {
    (vec.x * vec.x + vec.y * vec.y + vec.z * vec.z).sqrt()
}

impl Ray {
    pub fn from_mouse_coords(mouse_coords: Vec2, window_size: Vec2, camera: &Camera, max_distance: f32) -> Ray {
        let device_coords = convert_screen_coords_to_device_coords(mouse_coords, window_size);
        let ray_clip = Vec4::new(device_coords.x, device_coords.y, -1.0, 1.0);
        let mut ray_eye = inverse(camera.get_projection()) * ray_clip;
        ray_eye = Vec4::new(ray_eye.x, ray_eye.y, -1.0, 0.0);
        let mut ray_world = inverse(camera.get_view()) * ray_eye;
        ray_world = Vec4::new(ray_world.x, ray_world.y, ray_world.z, 0.0);
        let direction = ray_world.normalize();
        Ray {
            origin: camera.get_position(),
            direction: *direction.deref(),
            max_distance,
        }
    }

    pub fn get_point(&self, distance: f32) -> Vec3 {
        self.origin + xyz(self.direction * distance)
    }

    pub fn is_colliding_and_how_far(&self, point_a: Vec3, point_b: Vec3) -> Option<f32> {
        let line = point_b - point_a;
        let line_length = length(line);
        let line_direction = line / line_length;
        let line_to_ray_origin = self.origin - point_a;
        let line_to_ray_origin_length = length(line_to_ray_origin);
        let line_to_ray_origin_direction = line_to_ray_origin / line_to_ray_origin_length;
        let dot = line_direction.dot(line_to_ray_origin_direction);
        let angle = dot.acos();
        let distance_to_line = line_to_ray_origin_length * angle.sin();
        if distance_to_line > 0.1 {
            return None;
        }
        let distance_to_ray_origin = line_to_ray_origin_length * angle.cos();
        if distance_to_ray_origin > line_length {
            return None;
        }
        let distance_to_intersection = distance_to_ray_origin;
        if distance_to_intersection > self.max_distance {
            return None;
        }
        Some(distance_to_intersection)
    }
}