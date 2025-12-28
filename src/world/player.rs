// SPDX-FileCopyrightText: © 2025 - 2026 Elias Steininger <elias.st4600@gmail.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::world::world::World;
use glam::{IVec3, Vec3};
use std::collections::HashSet;
use winit::keyboard::KeyCode;

const PLAYER_HEIGHT: f32 = 1.8;
const MOUSE_SENS: f32 = 0.04;
const GRAVITY: u8 = 23;
const JUMP_STRENGTH: u8 = 8;
const SPEED: u8 = 4;
const SPRINT_SPEED: u8 = 6;

pub struct Player {
    pos: Vec3,
    camera_front: Vec3,
    yaw: f32,
    pitch: f32,
    vertical_velocity: f32,
    was_grounded: bool,
    health: u8
}

impl Player {
    pub fn new() -> Self {
        Player {
            pos: Vec3::new(2.0, 125.0, 2.0),
            camera_front: Vec3::new(0.0, 0.0, -1.0),
            yaw: -90.0,
            pitch: 0.0,
            vertical_velocity: 0.0,
            was_grounded: false,
            health: 8
        }
    }

    pub fn update_pos(&mut self, delta_time: f32, keys_pressed: HashSet<KeyCode>, world: &World) {

        if delta_time >= 0.3 { return; } // On game begin delta_time can be very big, which leads to weird behavior

        let is_grounded: bool = self.is_block_at(IVec3::new(self.pos.x.floor() as i32, self.pos.y.floor() as i32, self.pos.z.floor() as i32), &world);
        let speed: f32 = if keys_pressed.contains(&KeyCode::ShiftLeft) {SPRINT_SPEED} else {SPEED} as f32 * delta_time;
        let camera_right: Vec3 = self.camera_front.cross(Vec3::Y).normalize();
        let camera_horizontal_front: Vec3 = Vec3::new(self.camera_front.x, 0.0, self.camera_front.z).normalize();

        let mut move_dir: Vec3 = Vec3::ZERO;
        let mut new_pos: Vec3 = self.pos;

        if !self.was_grounded && is_grounded {
            let damage: f32 = (-self.vertical_velocity * 0.4) - 5.0;

            if damage > 0.0 {
                self.damage(damage.floor() as u8);
            }
        }

        self.was_grounded = is_grounded;

        if !is_grounded {
            self.vertical_velocity -= GRAVITY as f32 * delta_time;
        } else {
            self.vertical_velocity = 0.0;
        }

        if keys_pressed.contains(&KeyCode::Space) && is_grounded {
            self.vertical_velocity += JUMP_STRENGTH as f32;
        }

        if keys_pressed.contains(&KeyCode::KeyW){
            move_dir += speed * camera_horizontal_front;
        }
        if keys_pressed.contains(&KeyCode::KeyS){
            move_dir -= speed * camera_horizontal_front;
        }
        if keys_pressed.contains(&KeyCode::KeyA){
            move_dir -= speed * camera_right;
        }
        if keys_pressed.contains(&KeyCode::KeyD){
            move_dir += speed * camera_right;
        }

        if move_dir.x != 0.0 {

            let mut offset: f32 = 0.5;

            if move_dir.x < 0.0 {
                offset = -0.5;
            }

            let block_foot: IVec3 = IVec3::new((self.pos.x + offset).floor() as i32, (self.pos.y + 0.5).floor() as i32, self.pos.z.floor() as i32);
            let block_head: IVec3 = IVec3::new((self.pos.x + offset).floor() as i32, (self.pos.y + 1.5).floor() as i32, self.pos.z.floor() as i32);

            if self.is_block_at(block_head, &world) || self.is_block_at(block_foot, &world) {
                move_dir.x = 0.0;
            }
        }

        if self.vertical_velocity > 0.0 {
            let block_head: IVec3 = IVec3::new(self.pos.x.floor() as i32, (self.pos.y + (PLAYER_HEIGHT + 0.15)).floor() as i32, self.pos.z.floor() as i32);

            if self.is_block_at(block_head, &world) {
                self.vertical_velocity = 0.0;
            }
        }

        if move_dir.z != 0.0 {

            let mut offset: f32 = 0.5;

            if move_dir.z < 0.0 {
                offset = -0.5;
            }

            let block_foot: IVec3 = IVec3::new(self.pos.x.floor() as i32, (self.pos.y + 0.5).floor() as i32, (self.pos.z + offset).floor() as i32);
            let block_head: IVec3 = IVec3::new(self.pos.x.floor() as i32, (self.pos.y + 1.5).floor() as i32, (self.pos.z + offset).floor() as i32);

            if self.is_block_at(block_head, &world) || self.is_block_at(block_foot, &world) {
                move_dir.z = 0.0;
            }
        }

        new_pos += Vec3::new(0.0, self.vertical_velocity * delta_time, 0.0) + move_dir;

        self.pos = new_pos;
    }

    fn is_block_at(&self, world_pos: IVec3, world: &World) -> bool {
        world.get_block(world_pos) != 0
    }

    pub fn update_rotation(&mut self, delta: (f64, f64)) {
        self.yaw += delta.0 as f32 * MOUSE_SENS;
        self.pitch -= delta.1 as f32 * MOUSE_SENS;
        self.pitch = self.pitch.clamp(-89.0, 89.0);

        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        self.camera_front = Vec3::new(yaw_rad.cos() * pitch_rad.cos(), pitch_rad.sin(), yaw_rad.sin() * pitch_rad.cos()).normalize();
    }

    pub fn get_pos(&self) -> Vec3 {self.pos}

    pub fn get_head_pos(&self) -> Vec3 {Vec3::new(self.pos.x, self.pos.y + PLAYER_HEIGHT, self.pos.z)}

    pub fn get_camera_front(&self) -> Vec3 {self.camera_front}
    
    pub fn get_health(&self) -> u8 {self.health}

    pub fn damage(&mut self, damage: u8) {
        if (self.health as i8 - damage as i8) <= 0 {
            self.health = 0;
        } else {
            self.health -= damage
        }
    }
}
