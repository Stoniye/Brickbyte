// SPDX-FileCopyrightText: © 2025 - 2026 Elias Steininger <elias.st4600@gmail.com> and Project Contributors (see CONTRIBUTORS.md)
// SPDX-License-Identifier: GPL-3.0-or-later

use glam::{IVec2, IVec3, Mat4, Vec3};
use glow::{Context, HasContext, NativeBuffer, NativeTexture, NativeVertexArray, Program};

pub const CHUNK_DIMENSION: u8 = 16;
pub const CHUNK_HEIGHT: u8 = 208;

pub struct Chunk {
    blocks: Vec<u8>,
    pub light_map: Vec<u8>,
    pub position: IVec2,
    shader: Program,
    pub vertices: Option<Vec<f32>>,
    pub indices: Option<Vec<i32>>,
    pub vertex_array_object: Option<NativeVertexArray>,
    pub vertex_buffer_object: Option<NativeBuffer>,
    pub element_buffer_object: Option<NativeBuffer>
}

impl Chunk {
    pub fn new(position: IVec2, shader: Program) -> Self {
        Chunk {
            blocks: vec![0; (CHUNK_DIMENSION as usize) * (CHUNK_HEIGHT as usize) * (CHUNK_DIMENSION as usize)],
            light_map: vec![0; (CHUNK_DIMENSION as usize) * (CHUNK_HEIGHT as usize) * (CHUNK_DIMENSION as usize)],
            position,
            shader,
            vertices: None,
            indices: None,
            vertex_array_object: None,
            vertex_buffer_object: None,
            element_buffer_object: None
        }
    }

    pub fn get_light(&self, pos: IVec3) -> u8 {
        if pos.x < 0 || pos.x >= CHUNK_DIMENSION as i32 || pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 || pos.z < 0 || pos.z >= CHUNK_DIMENSION as i32 {
            return 15;
        }
        self.light_map[Self::get_block_index(pos)]
    }

    pub fn set_light(&mut self, pos: IVec3, level: u8) {
        if pos.x >= 0 && pos.x < CHUNK_DIMENSION as i32 && pos.y >= 0 && pos.y < CHUNK_HEIGHT as i32 && pos.z >= 0 && pos.z < CHUNK_DIMENSION as i32 {
            self.light_map[Self::get_block_index(pos)] = level;
        }
    }
    
    pub fn setup_buffers(&mut self, gl: &Context) {
        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, self.vertices.as_ref().unwrap().align_to::<u8>().1, glow::STATIC_DRAW);

            const STRIDE: i32 = 6 * size_of::<f32>() as i32;
            
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, STRIDE, 0);
            gl.enable_vertex_attrib_array(0);
            
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, STRIDE, 3 * size_of::<f32>() as i32);
            gl.enable_vertex_attrib_array(1);

            gl.vertex_attrib_pointer_f32(2, 1, glow::FLOAT, false, STRIDE, 5 * size_of::<f32>() as i32);
            gl.enable_vertex_attrib_array(2);
            
            let ebo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, self.indices.as_ref().unwrap().align_to::<u8>().1, glow::STATIC_DRAW);
            
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            
            self.vertex_array_object = Some(vao);
            self.vertex_buffer_object = Some(vbo);
            self.element_buffer_object = Some(ebo);
        }
    }
    
    pub fn render(&self, gl: &Context, pv: Mat4, texture: Option<NativeTexture>) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, texture);
            
            let model = Mat4::from_translation(Vec3::new((self.position.x as f32) * CHUNK_DIMENSION as f32, 0.0, (self.position.y as f32) * CHUNK_DIMENSION as f32));
            let mvp = pv * model;
            
            gl.use_program(Some(self.shader));
            gl.uniform_matrix_4_f32_slice(gl.get_uniform_location(self.shader, "mvp").as_ref(), false, mvp.as_ref());
            
            gl.bind_vertex_array(self.vertex_array_object);
            gl.draw_elements(glow::TRIANGLES, self.indices.as_ref().unwrap().len() as i32, glow::UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);
        }
    }

    fn get_block_index(pos: IVec3) -> usize {
        (pos.x + pos.z * CHUNK_DIMENSION as i32 + pos.y * (CHUNK_DIMENSION as i32 * CHUNK_DIMENSION as i32)) as usize
    }

    pub fn get_block(&self, pos: IVec3) -> u8 {
        if pos.x < 0 || pos.x >= CHUNK_DIMENSION as i32 || pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 || pos.z < 0 || pos.z >= CHUNK_DIMENSION as i32 {
            return 0;
        }

        self.blocks[Self::get_block_index(pos)]
    }

    pub fn set_block(&mut self, pos: IVec3, id: u8) {
        if pos.x >= 0 && pos.x < CHUNK_DIMENSION as i32 && pos.y >= 0 && pos.y < CHUNK_HEIGHT as i32 && pos.z >= 0 && pos.z < CHUNK_DIMENSION as i32 {
            self.blocks[Self::get_block_index(pos)] = id;
        }
    }
}
