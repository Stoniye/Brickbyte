use crate::world::chunk::{Chunk, CHUNK_DIMENSION};
use glam::{IVec2, IVec3, Mat4, Vec3};
use std::collections::HashMap;
use glow::{Context, NativeTexture, Program};

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
    
    pub fn insert_chunk(&mut self, pos: IVec2, shader: Program) {
        self.chunks.insert(IVec2::new(pos.x, pos.y), Chunk::new(IVec2::new(pos.x, pos.y), shader));
    }
    
    pub fn reload_world(&mut self, gl: &Context) {
        for (_pos, chunk) in self.chunks.iter_mut(){
            chunk.reload_chunk(gl);
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
    
    pub fn get_block(&self, world_pos: IVec3) -> u8 {
        let (chunk_pos, block_pos) = Self::world_to_local(world_pos);
        
        if self.chunks.contains_key(&chunk_pos) {
            return self.chunks.get(&chunk_pos).unwrap().get_block(block_pos);
        }
        
        0
    }
    
    pub fn set_block(&mut self, world_pos: IVec3, id: u8, gl: &Context) {
        let (chunk_pos, block_pos) = Self::world_to_local(world_pos);

        self.chunks.get_mut(&chunk_pos).unwrap().set_block(block_pos, id);
        self.chunks.get_mut(&chunk_pos).unwrap().reload_chunk(gl);
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
            if self.get_block(pos) != 0 {
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
}
