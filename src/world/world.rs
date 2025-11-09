use std::cell::RefCell;
use crate::world::chunk::Chunk;
use glam::{IVec2, Mat4};
use std::collections::HashMap;
use std::rc::Weak;
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

    pub fn insert_chunk(&mut self, pos: IVec2, shader: Program, world_weak: Weak<RefCell<World>>) {
        self.chunks.insert(pos, Chunk::new(pos, shader, world_weak));
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
}