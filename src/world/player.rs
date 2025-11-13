use std::collections::HashSet;
use glam::Vec3;
use winit::keyboard::KeyCode;

pub struct Player {
    pos: Vec3,
    speed: f32,
    camera_front: Vec3,
    camera_up: Vec3,
    yaw: f32,
    pitch: f32,
    mouse_sens: f32,
}

impl Player {
    pub fn new() -> Self {
        Player {
            pos: Vec3::new(0.0, 0.0, 0.0),
            speed: 2.5,
            camera_front: Vec3::new(0.0, 0.0, -1.0),
            camera_up: Vec3::Y,
            yaw: -90.0,
            pitch: 0.0,
            mouse_sens: 0.04,
        }
    }

    pub fn update_pos(&mut self, delta_time: f32, keys_pressed: HashSet<KeyCode>) {
        let camera_speed = self.speed * delta_time;
        let camera_right = self.camera_front.cross(self.camera_up).normalize();

        if keys_pressed.contains(&KeyCode::KeyW){
            self.pos += camera_speed * self.camera_front;
        }
        if keys_pressed.contains(&KeyCode::KeyS){
            self.pos -= camera_speed * self.camera_front;
        }
        if keys_pressed.contains(&KeyCode::KeyA){
            self.pos -= camera_speed * camera_right;
        }
        if keys_pressed.contains(&KeyCode::KeyD){
            self.pos += camera_speed * camera_right;
        }
        if keys_pressed.contains(&KeyCode::ShiftLeft){
            self.pos -= camera_speed * self.camera_up;
        }
        if keys_pressed.contains(&KeyCode::Space){
            self.pos += camera_speed * self.camera_up;
        }
    }

    pub fn update_rotation(&mut self, delta: (f64, f64)) {
        self.yaw += delta.0 as f32 * self.mouse_sens;
        self.pitch -= delta.1 as f32 * self.mouse_sens;
        self.pitch = self.pitch.clamp(-89.0, 89.0);

        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        self.camera_front = Vec3::new(yaw_rad.cos() * pitch_rad.cos(), pitch_rad.sin(), yaw_rad.sin() * pitch_rad.cos()).normalize();
    }

    pub fn get_pos(&self) -> Vec3 {self.pos}

    pub fn get_camera_front(&self) -> Vec3 {self.camera_front}

    pub fn get_camera_up(&self) -> Vec3 {self.camera_up}
}