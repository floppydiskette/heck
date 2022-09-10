use std::io::Read;
use gfx_maths::{Quaternion, Vec3};
use crate::renderer::types::Colour;

pub fn gen_rainbow(time: f64) -> Colour {
    let frequency = 0.05;
    let r = ((frequency * (time as f64) + 0.0).sin() * 127.0f64 + 128.0f64);
    let g = ((frequency * (time as f64) + 2.0).sin() * 127.0f64 + 128.0f64);
    let b = ((frequency * (time as f64) + 4.0).sin() * 127.0f64 + 128.0f64);
    Colour { r: (r) as u8, g: (g) as u8, b: (b) as u8, a: 255 }
}

pub fn load_string_from_file(path: String) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|e| e.to_string())?;
    Ok(contents)
}

pub fn get_quaternion_yaw(quat: Quaternion) -> f32 {
    let mut yaw = 0.0;
    let test = quat.x * quat.y + quat.z * quat.w;
    if test > 0.499 {
        yaw = 2.0 * (quat.x).atan2(quat.w);
    } else if test < -0.499 {
        yaw = -2.0 * (quat.x).atan2(quat.w);
    } else {
        let sqx = quat.x * quat.x;
        let sqy = quat.y * quat.y;
        let sqz = quat.z * quat.z;
        yaw = (sqy + sqx - sqz - quat.w * quat.w).atan2(2.0 * quat.y * quat.x + 2.0 * quat.z * quat.w);
    }
    yaw
}

pub fn get_quaternion_pitch(quat: Quaternion) -> f32 {
    let mut pitch = 0.0;
    let test = quat.x * quat.y + quat.z * quat.w;
    if test > 0.499 {
        pitch = std::f32::consts::PI / 2.0;
    } else if test < -0.499 {
        pitch = -std::f32::consts::PI / 2.0;
    } else {
        let sqx = quat.x * quat.x;
        let sqy = quat.y * quat.y;
        let sqz = quat.z * quat.z;
        pitch = (sqz - sqx - sqy + quat.w * quat.w).atan2(2.0 * quat.z * quat.y + 2.0 * quat.x * quat.w);
    }
    pitch
}

pub fn yaw_pitch_to_quaternion(yaw: f32, pitch: f32) -> Quaternion {
    Quaternion::from_euler_angles_zyx(&Vec3::new(pitch, yaw, 0.0))
}

pub fn conjugate_quaternion(quat: Quaternion) -> Quaternion {
    Quaternion::new(-quat.x, -quat.y, -quat.z, quat.w)
}

pub fn rotate_vector_by_quaternion(vector: Vec3, quat: Quaternion) -> Vec3 {
    let mut quat_v = Quaternion::new(vector.x, vector.y, vector.z, 0.0);
    quat_v = quat_v * quat;
    quat_v = conjugate_quaternion(quat) * quat_v;
    Vec3::new(quat_v.x, quat_v.y, quat_v.z)
}

// returns the corrected uvs in the same order as the vertices
pub fn fix_colladas_dumb_storage_method(uv_array: Vec<f32>, uv_indices: Vec<u32>) -> Vec<f32> {
    let mut uvs = Vec::new();
    for i in 0..uv_indices.len() {
        let index = uv_indices[i] as usize;
        uvs.push(uv_array[index]);

    }
    uv_array
}