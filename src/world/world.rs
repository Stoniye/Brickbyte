// SPDX-FileCopyrightText: © 2025 - 2026 Elias Steininger <elias.st4600@gmail.com> and Project Contributors (see CONTRIBUTORS.md)
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::world::chunk::{Chunk, CHUNK_DIMENSION, CHUNK_HEIGHT};
use glam::{IVec2, IVec3, Mat4, Vec2, Vec3};
use glow::{Context, HasContext, NativeTexture, Program};
use std::collections::{HashMap, VecDeque};
use rand::Rng;

pub struct World {
    chunks: HashMap<IVec2, Chunk>,
}

pub struct BlockRaycast {
    pub block_pos: IVec3,
    pub prev_block_pos: IVec3
}

impl World {
    pub fn new() -> Self {
        World {
            chunks: HashMap::new()
        }
    }
    
    pub fn insert_chunk(&mut self, pos: IVec2, shader: Program, noise_map: Vec<Vec<f64>>) {
        let mut chunk = Chunk::new(IVec2::new(pos.x, pos.y), shader);

        Self::generate_chunk(&mut chunk, noise_map);
        self.calculate_chunk_lighting(&mut chunk);

        self.chunks.insert(IVec2::new(pos.x, pos.y), chunk);
    }

    pub fn reload_world(&mut self, gl: &Context) {
        let positions: Vec<IVec2> = self.chunks.keys().cloned().collect();

        for pos in positions {
            if let Some(mut chunk) = self.chunks.remove(&pos) {
                self.generate_chunk_mesh(&mut chunk, gl);

                self.chunks.insert(pos, chunk);
            }
        }
    }
    
    pub fn render_world(&mut self, gl: &Context, pv: Mat4, texture: Option<NativeTexture>) {
        for (_pos, chunk) in &self.chunks {
            chunk.render(gl, pv, texture);
        }
    }
    
    fn world_to_local(world_pos: IVec3) -> (IVec2, IVec3) {
        let chunk_pos = IVec2::new(world_pos.x.div_euclid(CHUNK_DIMENSION as i32), world_pos.z.div_euclid(CHUNK_DIMENSION as i32));
        let block_pos = IVec3::new(world_pos.x.rem_euclid(CHUNK_DIMENSION as i32), world_pos.y, world_pos.z.rem_euclid(CHUNK_DIMENSION as i32));
        
        (chunk_pos, block_pos)
    }
    
    pub fn get_global_block(&self, world_pos: IVec3) -> u8 {
        let (chunk_pos, block_pos) = Self::world_to_local(world_pos);
        
        if self.chunks.contains_key(&chunk_pos) {
            return self.chunks.get(&chunk_pos).unwrap().get_block(block_pos);
        }
        
        0
    }

    pub fn set_block(&mut self, world_pos: IVec3, id: u8, gl: &Context) {
        let (chunk_pos, block_pos) = Self::world_to_local(world_pos);

        let mut chunks_to_reload = vec![chunk_pos];

        if block_pos.x == 0 {chunks_to_reload.push(chunk_pos - IVec2::X);}
        if block_pos.x == CHUNK_DIMENSION as i32 - 1 {chunks_to_reload.push(chunk_pos + IVec2::X);}
        if block_pos.z == 0 {chunks_to_reload.push(chunk_pos - IVec2::Y);}
        if block_pos.z == CHUNK_DIMENSION as i32 - 1 {chunks_to_reload.push(chunk_pos + IVec2::Y);}

        for c_pos in chunks_to_reload {
            if let Some(mut chunk) = self.chunks.remove(&c_pos) {
                if c_pos == chunk_pos {
                    chunk.set_block(block_pos, id);
                }

                self.calculate_chunk_lighting(&mut chunk);
                self.generate_chunk_mesh(&mut chunk, gl);
                self.chunks.insert(c_pos, chunk);
            }
        }
    }
    
    pub fn raycast_block(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<BlockRaycast> {
        let mut pos = origin.floor().as_ivec3();
        let step = IVec3::new(
            if direction.x < 0.0 { -1 } else { 1 },
            if direction.y < 0.0 { -1 } else { 1 },
            if direction.z < 0.0 { -1 } else { 1 },
        );
        let t_delta = Vec3::new(
            (1.0 / direction.x.abs()).min(f32::MAX),
            (1.0 / direction.y.abs()).min(f32::MAX),
            (1.0 / direction.z.abs()).min(f32::MAX),
        );
        let mut t_max = Vec3::new(
            if direction.x < 0.0 { (origin.x - pos.x as f32) / direction.x } else { ((pos.x + 1) as f32 - origin.x) / direction.x },
            if direction.y < 0.0 { (origin.y - pos.y as f32) / direction.y } else { ((pos.y + 1) as f32 - origin.y) / direction.y },
            if direction.z < 0.0 { (origin.z - pos.z as f32) / direction.z } else { ((pos.z + 1) as f32 - origin.z) / direction.z },
        ).abs();
        
        let mut distance = 0.0;
        let mut face_normal = IVec3::ZERO;
        
        while distance < max_distance {
            if self.get_global_block(pos) != 0 {
                return Some(BlockRaycast {
                    block_pos: pos,
                    prev_block_pos: pos + face_normal
                });
            }
            
            if t_max.x < t_max.y && t_max.x < t_max.z {
                pos.x += step.x;
                distance = t_max.x;
                t_max.x += t_delta.x;
                face_normal = IVec3::new(-step.x, 0, 0);
            } else if t_max.y < t_max.z {
                pos.y += step.y;
                distance = t_max.y;
                t_max.y += t_delta.y;
                face_normal = IVec3::new(0, -step.y, 0);
            } else {
                pos.z += step.z;
                distance = t_max.z;
                t_max.z += t_delta.z;
                face_normal = IVec3::new(0, 0, -step.z);
            }
        }
        
        None
    }

    fn generate_chunk(chunk: &mut Chunk, noise_map: Vec<Vec<f64>>) {
        for x in 0..CHUNK_DIMENSION {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_DIMENSION {
                    let y_map: f64 = 50.0 + (noise_map[z as usize][x as usize] * 100.0); // 50 <= y_map <= 150
                    let stone_y: f64 = rand::rng().random_range(3..6) as f64;

                    if y as f64 <= (y_map - stone_y) {
                        chunk.set_block(IVec3::new(x as i32, y as i32, z as i32), 3);
                    } else if y as f64 <= (y_map - 1.0) {
                        chunk.set_block(IVec3::new(x as i32, y as i32, z as i32), 2);
                    } else if y as f64 <= y_map {
                        chunk.set_block(IVec3::new(x as i32, y as i32, z as i32), 1);
                    }
                }
            }
        }
    }

    fn calculate_chunk_lighting(&self, chunk: &mut Chunk) {
        let mut queue: VecDeque<IVec3> = VecDeque::new();
        chunk.light_map.fill(0);

        //Set every air block to light level 15 from top to down (sunlight)
        for x in 0..CHUNK_DIMENSION as i32 {
            for z in 0..CHUNK_DIMENSION as i32 {
                for y in (0..CHUNK_HEIGHT as i32).rev() {
                    let pos = IVec3::new(x, y, z);
                    if !self.block_is_air(chunk, pos) {
                        break;
                    }
                    chunk.set_light(pos, 15);
                    queue.push_back(pos);
                }
            }
        }

        //Handle light propagation on blocks
        while let Some(pos) = queue.pop_front() {
            let current_light = chunk.get_light(pos);
            if current_light <= 1 { continue; }

            let neighbors = [
                IVec3::new(1, 0, 0), IVec3::new(-1, 0, 0),
                IVec3::new(0, 1, 0), IVec3::new(0, -1, 0),
                IVec3::new(0, 0, 1), IVec3::new(0, 0, -1),
            ];

            for offset in neighbors {
                let neighbor_pos = pos + offset;

                if self.block_is_air(chunk, neighbor_pos) {
                    let neighbor_light = chunk.get_light(neighbor_pos);

                    if neighbor_light < current_light - 1 {
                        chunk.set_light(neighbor_pos, current_light - 1);
                        queue.push_back(neighbor_pos);
                    }
                }
            }
        }
    }

    fn generate_chunk_mesh(&self, chunk: &mut Chunk, gl: &Context){
        unsafe {
            if let Some(vao) = chunk.vertex_array_object {
                gl.delete_vertex_array(vao);
            }
            if let Some(vbo) = chunk.vertex_buffer_object {
                gl.delete_buffer(vbo);
            }
            if let Some(ebo) = chunk.element_buffer_object {
                gl.delete_buffer(ebo);
            }
        }

        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<i32> = Vec::new();

        let mut index: i32 = 0;

        for y in 0..CHUNK_HEIGHT as i32 {
            for z in 0..CHUNK_DIMENSION as i32 {
                for x in 0..CHUNK_DIMENSION as i32 {

                    let block_pos = IVec3::new(x, y, z);
                    let block_type = chunk.get_block(IVec3::new(x, y, z));

                    // Skip air
                    if block_type == 0 { continue; }

                    let texture_coords = Self::get_texture_coords(&block_type);

                    // Front face (Z + 1)
                    if self.block_is_air(chunk, IVec3::new(x, y, z + 1)) {
                        self.add_face(chunk, &mut vertices, &mut indices, block_pos, IVec3::new(0, 0, 1), &mut index, texture_coords);
                    }

                    // Back Face (Z - 1)
                    if self.block_is_air(chunk, IVec3::new(x, y, z - 1)) {
                        self.add_face(chunk, &mut vertices, &mut indices, block_pos, IVec3::new(0, 0, -1), &mut index, texture_coords);
                    }

                    // Top Face (Y + 1)
                    if self.block_is_air(chunk, IVec3::new(x, y + 1, z)) {
                        self.add_face(chunk, &mut vertices, &mut indices, block_pos, IVec3::new(0, 1, 0), &mut index, texture_coords);
                    }

                    // Bottom Face (Y - 1)
                    if self.block_is_air(chunk, IVec3::new(x, y - 1, z)) {
                        self.add_face(chunk, &mut vertices, &mut indices, block_pos, IVec3::new(0, -1, 0), &mut index, texture_coords);
                    }

                    // Left Face (X - 1)
                    if self.block_is_air(chunk, IVec3::new(x - 1, y, z)) {
                        self.add_face(chunk, &mut vertices, &mut indices, block_pos, IVec3::new(-1, 0, 0), &mut index, texture_coords);
                    }

                    // Right Face (X + 1)
                    if self.block_is_air(chunk, IVec3::new(x + 1, y, z)) {
                        self.add_face(chunk, &mut vertices, &mut indices, block_pos, IVec3::new(1, 0, 0), &mut index, texture_coords);
                    }
                }
            }
        }

        chunk.vertices = Some(vertices);
        chunk.indices = Some(indices);

        chunk.setup_buffers(&gl);
    }

    fn add_face(&self, chunk: &Chunk, vertices: &mut Vec<f32>, indices: &mut Vec<i32>, pos: IVec3, normal: IVec3, index: &mut i32, texture_coords: [Vec2; 4]) {
        let pos_float: Vec3 = Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5);
        let mut face_vertices: [Vec3; 4] = [Vec3::ZERO; 4];
        let mut face_light: [f32; 4] = [1.0; 4];
        let light_level = chunk.get_light(pos + normal);
        let sunlight_brightness = light_level as f32 / 15.0;

        let face_brightness = match normal {
            IVec3 { x: 0, y: 1, z: 0 } => 1.0, // Top (Sunlight)
            IVec3 { x: 0, y: -1, z: 0 } => 0.4, // Bottom
            IVec3 { x: 1, y: 0, z: 0 } | IVec3 { x: -1, y: 0, z: 0 } => 0.8, // X-side
            IVec3 { x: 0, y: 0, z: 1 } | IVec3 { x: 0, y: 0, z: -1 } => 0.8, // Z-side
            _ => 1.0,
        };

        match normal {

            // Front Face
            IVec3 { x: 0, y: 0, z: 1 } => {
                face_vertices[0] = pos_float + Vec3::new(-0.5, -0.5, 0.5);
                face_vertices[1] = pos_float + Vec3::new(0.5, -0.5, 0.5);
                face_vertices[2] = pos_float + Vec3::new(0.5, 0.5, 0.5);
                face_vertices[3] = pos_float + Vec3::new(-0.5, 0.5, 0.5);

                let adjacent = pos + normal;
                face_light[0] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(-1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,-1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(-1,-1,0)));
                face_light[1] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,-1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(1,-1,0)));
                face_light[2] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(1,1,0)));
                face_light[3] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(-1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(-1,1,0)));
            }

            // Back Face
            IVec3 { x: 0, y: 0, z: -1 } => {
                face_vertices[0] = pos_float + Vec3::new(0.5, -0.5, -0.5);
                face_vertices[1] = pos_float + Vec3::new(-0.5, -0.5, -0.5);
                face_vertices[2] = pos_float + Vec3::new(-0.5, 0.5, -0.5);
                face_vertices[3] = pos_float + Vec3::new(0.5, 0.5, -0.5);

                let adjacent = pos + normal;
                face_light[0] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,-1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(1,-1,0)));
                face_light[1] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(-1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,-1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(-1,-1,0)));
                face_light[2] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(-1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(-1,1,0)));
                face_light[3] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(1,1,0)));
            }

            // Top Face
            IVec3 { x: 0, y: 1, z: 0 } => {
                face_vertices[0] = pos_float + Vec3::new(-0.5, 0.5, -0.5);
                face_vertices[1] = pos_float + Vec3::new(0.5, 0.5, -0.5);
                face_vertices[2] = pos_float + Vec3::new(0.5, 0.5, 0.5);
                face_vertices[3] = pos_float + Vec3::new(-0.5, 0.5, 0.5);

                let adjacent = pos + normal;
                face_light[0] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(-1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,-1)), !self.block_is_air(chunk, adjacent + IVec3::new(-1,0,-1)));
                face_light[1] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,-1)), !self.block_is_air(chunk, adjacent + IVec3::new(1,0,-1)));
                face_light[2] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,1)), !self.block_is_air(chunk, adjacent + IVec3::new(1,0,1)));
                face_light[3] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(-1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,1)), !self.block_is_air(chunk, adjacent + IVec3::new(-1,0,1)));
            }

            // Bottom Face
            IVec3 { x: 0, y: -1, z: 0 } => {
                face_vertices[0] = pos_float + Vec3::new(-0.5, -0.5, 0.5);
                face_vertices[1] = pos_float + Vec3::new(0.5, -0.5, 0.5);
                face_vertices[2] = pos_float + Vec3::new(0.5, -0.5, -0.5);
                face_vertices[3] = pos_float + Vec3::new(-0.5, -0.5, -0.5);

                let adjacent = pos + normal;
                face_light[0] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(-1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,1)), !self.block_is_air(chunk, adjacent + IVec3::new(-1,0,1)));
                face_light[1] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,1)), !self.block_is_air(chunk, adjacent + IVec3::new(1,0,1)));
                face_light[2] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,-1)), !self.block_is_air(chunk, adjacent + IVec3::new(1,0,-1)));
                face_light[3] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(-1,0,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,-1)), !self.block_is_air(chunk, adjacent + IVec3::new(-1,0,-1)));
            }

            // Left Face
            IVec3 { x: -1, y: 0, z: 0 } => {
                face_vertices[0] = pos_float + Vec3::new(-0.5, -0.5, -0.5);
                face_vertices[1] = pos_float + Vec3::new(-0.5, -0.5, 0.5);
                face_vertices[2] = pos_float + Vec3::new(-0.5, 0.5, 0.5);
                face_vertices[3] = pos_float + Vec3::new(-0.5, 0.5, -0.5);

                let adjacent = pos + normal;
                face_light[0] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(0,-1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,-1)), !self.block_is_air(chunk, adjacent + IVec3::new(0,-1,-1)));
                face_light[1] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(0,-1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,1)), !self.block_is_air(chunk, adjacent + IVec3::new(0,-1,1)));
                face_light[2] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(0,1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,1)), !self.block_is_air(chunk, adjacent + IVec3::new(0,1,1)));
                face_light[3] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(0,1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,-1)), !self.block_is_air(chunk, adjacent + IVec3::new(0,1,-1)));
            }

            // Right Face
            IVec3 { x: 1, y: 0, z: 0 } => {
                face_vertices[0] = pos_float + Vec3::new(0.5, -0.5, 0.5);
                face_vertices[1] = pos_float + Vec3::new(0.5, -0.5, -0.5);
                face_vertices[2] = pos_float + Vec3::new(0.5, 0.5, -0.5);
                face_vertices[3] = pos_float + Vec3::new(0.5, 0.5, 0.5);

                let adjacent = pos + normal;
                face_light[0] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(0,-1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,1)), !self.block_is_air(chunk, adjacent + IVec3::new(0,-1,1)));
                face_light[1] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(0,-1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,-1)), !self.block_is_air(chunk, adjacent + IVec3::new(0,-1,-1)));
                face_light[2] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(0,1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,-1)), !self.block_is_air(chunk, adjacent + IVec3::new(0,1,-1)));
                face_light[3] = Self::vertex_ao(!self.block_is_air(chunk, adjacent + IVec3::new(0,1,0)), !self.block_is_air(chunk, adjacent + IVec3::new(0,0,1)), !self.block_is_air(chunk, adjacent + IVec3::new(0,1,1)));
            }

            _ => {}
        }

        for i in 0..4 {
            vertices.push(face_vertices[i].x);
            vertices.push(face_vertices[i].y);
            vertices.push(face_vertices[i].z);
            vertices.push(texture_coords[i].x);
            vertices.push(texture_coords[i].y);
            vertices.push(face_light[i] * face_brightness * sunlight_brightness);
        }

        indices.push(*index + 0);
        indices.push(*index + 1);
        indices.push(*index + 2);
        indices.push(*index + 2);
        indices.push(*index + 3);
        indices.push(*index + 0);
        *index += 4;
    }

    fn get_texture_coords(block_type: &u8) -> [Vec2; 4] {
        let atlas_size: u8 = 16;
        let tile_size: f32 = 1.0 / atlas_size as f32;
        let index: u8 = block_type - 1;
        let text_x: u8 = index % atlas_size;
        let text_y: u8 = index / atlas_size;
        let x: f32 = text_x as f32 * tile_size;
        let y: f32 = text_y as f32 * tile_size;

        [
            Vec2::new(x, y),
            Vec2::new(x + tile_size, y),
            Vec2::new(x + tile_size, y + tile_size),
            Vec2::new(x, y + tile_size),
        ]
    }

    fn block_is_air(&self, chunk: &Chunk, pos: IVec3) -> bool {
        if pos.x >= 0 && pos.x < CHUNK_DIMENSION as i32 && pos.z >= 0 && pos.z < CHUNK_DIMENSION as i32 && pos.y >= 0 && pos.y < CHUNK_HEIGHT as i32 {
            return chunk.get_block(pos) == 0;
        }

        self.get_global_block(pos + IVec3::new(chunk.position.x * CHUNK_DIMENSION as i32, 0, chunk.position.y * CHUNK_DIMENSION as i32)) == 0
    }

    fn vertex_ao(side1: bool, side2: bool, corner: bool) -> f32 {
        let mut occlusion = 0;

        if side1 && side2 {
            occlusion = 3;
        } else {
            if side1 {occlusion += 1};
            if side2 {occlusion += 1};
            if corner {occlusion += 1};
        }

        1.0 - (occlusion as f32 * 0.25)
    }
}
