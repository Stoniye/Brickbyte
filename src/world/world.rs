use crate::world::chunk::Chunk;
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

    pub fn set_block(&mut self, world_pos: IVec3, gl: &Context) {
        let chunk_pos: IVec2 = IVec2::new(((world_pos.x / 16) as f32).floor() as i32, ((world_pos.z / 16) as f32).floor() as i32);
        let pos: IVec3 = IVec3::new(((world_pos.x % 16) + 16) % 16, world_pos.y, ((world_pos.z % 16) + 16) % 16);

        self.chunks.get_mut(&chunk_pos).unwrap().set_block(pos, 0);
        self.chunks.get_mut(&chunk_pos).unwrap().reload_chunk(gl);
    }
}