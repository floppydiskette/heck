use std::io::Read;
use gfx_maths::Quaternion;
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