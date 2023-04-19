use gfx_maths::{Quaternion, Vec2, Vec3};
use crate::{helpers, keyboard, mouse};
use crate::helpers::lerp;
use crate::keyboard::HTKey;
use crate::renderer::ht_renderer;

#[derive(Clone)]
pub struct Viewport {
    position: Vec3,
    rotation: Quaternion,
    pitch: f64,
    yaw: f64,
    last_mouse_pos: Option<Vec2>,
    movement_speed: f32,
    wasd: [bool; 4],
    locked_mouse: bool,
    sprinting: bool,
    pub has_camera_control: bool,
}

impl Viewport {
    pub fn init(position: Vec3, rotation: Quaternion) -> Self {
        let mut viewport = Viewport {
            position,
            rotation,
            pitch: 0.0,
            yaw: 0.0,
            last_mouse_pos: None,
            movement_speed: 0.15,
            wasd: [false; 4],
            locked_mouse: false,
            sprinting: false,
            has_camera_control: false,
        };
        viewport.calculate_pitch_and_yaw_from_rotation(rotation);
        viewport.position = position;
        viewport.rotation = rotation;
        viewport
    }

    fn handle_mouse_movement(&mut self, renderer: &mut ht_renderer, delta_time: f32) -> Option<Quaternion> {

        let mouse_pos = mouse::get_mouse_pos();

        if self.last_mouse_pos.is_none() {
            self.last_mouse_pos = Some(mouse_pos);
        }
        let last_mouse_pos = self.last_mouse_pos.unwrap();
        let ang_x = -(mouse_pos.x as f64 - last_mouse_pos.x as f64);
        let ang_y = -(mouse_pos.y as f64 - last_mouse_pos.y as f64);
        self.last_mouse_pos = Some(mouse_pos);

        if !self.has_camera_control {
            return None;
        }


        let camera = &mut renderer.camera;
        let camera_rotation = camera.get_rotation();
        let mut pitch = self.pitch;
        let mut yaw = self.yaw;
        let original_yaw = yaw;
        let original_pitch = pitch;
        yaw += ang_x;
        pitch += ang_y;

        if pitch > 89.0 {
            pitch = 89.0;
        }
        if pitch < -89.0 {
            pitch = -89.0;
        }
        if pitch > 360.0 {
            pitch -= 360.0;
        }

        if yaw > 360.0 {
            yaw -= 360.0;
        }

        self.pitch = pitch;
        self.yaw = yaw;

        yaw -= original_yaw;
        pitch -= original_pitch;


        let horiz = Quaternion::from_euler_angles_zyx(&Vec3::new(0.0, yaw as f32, 0.0));
        let vert = Quaternion::from_euler_angles_zyx(&Vec3::new(pitch as f32, 0.0, 0.0));

        let new_camera_rotation = vert * camera_rotation * horiz;

        camera.set_rotation(new_camera_rotation);

        self.set_rotation(new_camera_rotation);

        if camera.get_rotation() != camera_rotation {
            Some(camera.get_rotation())
        } else {
            None
        }
    }

    fn handle_keyboard_movement(&mut self, renderer: &mut ht_renderer, delta_time: f32) {

        pub const DEFAULT_MOVESPEED: f32 = 1.15;
        pub const DEFAULT_SPRINTSPEED: f32 = 17.4;
        pub const DEFAULT_FOV: f32 = 120.0;
        pub const SPRINT_FOV: f32 = 140.0;
        let mut movement = Vec3::new(0.0, 0.0, 0.0);
        let camera = &mut renderer.camera;
        let camera_rotation = camera.get_rotation();
        let camera_forward = camera.get_forward();
        let camera_right = camera.get_right();
        let camera_up = camera.get_up();

        if keyboard::check_key_pressed(HTKey::W) {
            self.wasd[0] = true;
        }
        if keyboard::check_key_released(HTKey::W) {
            self.wasd[0] = false;
        }
        if keyboard::check_key_pressed(HTKey::A) {
            self.wasd[1] = true;
        }
        if keyboard::check_key_released(HTKey::A) {
            self.wasd[1] = false;
        }
        if keyboard::check_key_pressed(HTKey::S) {
            self.wasd[2] = true;
        }
        if keyboard::check_key_released(HTKey::S) {
            self.wasd[2] = false;
        }
        if keyboard::check_key_pressed(HTKey::D) {
            self.wasd[3] = true;
        }
        if keyboard::check_key_released(HTKey::D) {
            self.wasd[3] = false;
        }
        if keyboard::check_key_down(HTKey::LeftShift) {
            self.sprinting = true;
        }
        if keyboard::check_key_released(HTKey::LeftShift) {
            self.sprinting = false;
        }
        if self.sprinting {
            self.movement_speed = DEFAULT_SPRINTSPEED;
        } else {
            self.movement_speed = DEFAULT_MOVESPEED;
        }
        if self.wasd[0] {
            movement += camera_forward;
        }
        if self.wasd[1] {
            movement += camera_right;
        }
        if self.wasd[2] {
            movement -= camera_forward;
        }
        if self.wasd[3] {
            movement -= camera_right;
        }
        movement = helpers::clamp_magnitude(movement, 1.0);

        movement *= self.movement_speed;
        if !self.has_camera_control {
            return;
        }


        self.set_position(camera.get_position() + movement * delta_time);
        *crate::ui::DEBUG_LOCATION.lock().unwrap() = self.get_position();
        camera.set_position(self.get_position());
    }

    pub fn handle_input(&mut self, renderer: &mut ht_renderer, delta_time: f32) {
        self.handle_mouse_movement(renderer, delta_time);
        self.handle_keyboard_movement(renderer, delta_time);
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn get_rotation(&mut self) -> Quaternion {
        self.rotation
    }

    fn calculate_pitch_and_yaw_from_rotation(&mut self, rotation: Quaternion) {
        let rotation = rotation.to_euler_angles_zyx();
        // todo! make this do something
    }

    pub fn set_rotation(&mut self, rotation: Quaternion) {
        self.rotation = rotation;
        self.calculate_pitch_and_yaw_from_rotation(rotation);
    }
}