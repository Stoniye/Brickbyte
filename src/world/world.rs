use crate::world::chunk::{Chunk, CHUNK_DIMENSION};
use glam::{IVec2, IVec3, Mat4};
use std::collections::HashMap;
use glow::{Context, Program};

pub struct World {
    chunks: HashMap<IVec2, Chunk>,
}

impl World {
    pub fn new() -> Self {
        World {
            chunks: HashMap::new()
        }
    }

    pub fn insert_chunk(&mut self, pos: IVec2, shader: Program, gl: &Context) {
        self.chunks.insert(IVec2::new(pos.x, pos.y), Chunk::new(IVec2::new(pos.x, pos.y), shader, gl));
    }

    pub fn reload_world(&mut self, gl: &Context) {
        for (_pos, chunk) in self.chunks.iter_mut(){
            chunk.reload_chunk(gl);
        }
    }

    pub fn render_world(&mut self, gl: &Context, pv: Mat4) {
        for (_pos, chunk) in &self.chunks {
            chunk.render(gl, pv);
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

    pub fn set_block(&mut self, world_pos: IVec3, gl: &Context) {
        let (chunk_pos, block_pos) = Self::world_to_local(world_pos);

        self.chunks.get_mut(&chunk_pos).unwrap().set_block(block_pos, 0);
        self.chunks.get_mut(&chunk_pos).unwrap().reload_chunk(gl);
    }
}