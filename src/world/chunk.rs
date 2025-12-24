use glam::{IVec2, IVec3, Mat4, Vec2, Vec3};
use glow::{Context, HasContext, NativeBuffer, NativeTexture, NativeVertexArray, Program};
use rand::Rng;
use std::collections::HashMap;

pub const CHUNK_DIMENSION: u8 = 16;
pub const CHUNK_HEIGHT: u8 = 100;

pub struct Chunk {
    blocks: HashMap<IVec3, u8>,
    position: IVec2,
    shader: Program,
    vertices: Option<Vec<f32>>,
    indices: Option<Vec<i32>>,
    vertex_array_object: Option<NativeVertexArray>,
    vertex_buffer_object: Option<NativeBuffer>,
    element_buffer_object: Option<NativeBuffer>
}

impl Chunk {
    pub fn new(position: IVec2, shader: Program) -> Self {
        let mut chunk: Chunk = Chunk{
            blocks: HashMap::new(),
            position,
            shader,
            vertices: None,
            indices: None,
            vertex_array_object: None,
            vertex_buffer_object: None,
            element_buffer_object: None
        };
        chunk.initialize();
        
        chunk
    }
    
    fn initialize_blocks(&mut self) {
        for x in 0..CHUNK_DIMENSION {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_DIMENSION {
                    let stone_y: u8 = rand::rng().random_range(12..15);

                    if y == 0 {
                        self.set_block(IVec3::new(x as i32, y as i32, z as i32), 5);
                    } else if y <= stone_y {
                        self.set_block(IVec3::new(x as i32, y as i32, z as i32), 3);
                    } else if y <= 15 {
                        self.set_block(IVec3::new(x as i32, y as i32, z as i32), 2);
                    } else if y <= 16 {
                        self.set_block(IVec3::new(x as i32, y as i32, z as i32), 1);
                    }
                }
            }
        }
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
    
    pub fn reload_chunk(&mut self, gl: &Context){
        //TODO: Only reload changed blocks and not whole chunk
        
        unsafe {
            if let Some(vao) = self.vertex_array_object {
                gl.delete_vertex_array(vao);
            }
            if let Some(vbo) = self.vertex_buffer_object {
                gl.delete_buffer(vbo);
            }
            if let Some(ebo) = self.element_buffer_object {
                gl.delete_buffer(ebo);
            }
        }
        
        self.vertices = Some(Vec::new());
        self.indices = Some(Vec::new());
        
        let mut index: i32 = 0;
        
        for (pos, block_type) in &self.blocks {
            if *block_type == 0 { continue; }
            
            let texture_coords: [Vec2; 4] = Self::get_texture_coords(block_type);
            
            //Front face
            if self.block_is_air(IVec3::new(pos.x, pos.y, pos.z + 1)) {
                Self::add_face(self.vertices.as_mut().unwrap(), self.indices.as_mut().unwrap(), *pos, IVec3::new(0, 0, 1), &mut index, texture_coords);
            }
            
            //Back Face
            if self.block_is_air(IVec3::new(pos.x, pos.y, pos.z - 1)) {
                Self::add_face(self.vertices.as_mut().unwrap(), self.indices.as_mut().unwrap(), *pos, IVec3::new(0, 0, -1), &mut index, texture_coords);
            }
            
            //Top Face
            if self.block_is_air(IVec3::new(pos.x, pos.y + 1, pos.z)) {
                Self::add_face(self.vertices.as_mut().unwrap(), self.indices.as_mut().unwrap(), *pos, IVec3::new(0, 1, 0), &mut index, texture_coords);
            }
            
            //Bottom Face
            if self.block_is_air(IVec3::new(pos.x, pos.y - 1, pos.z)) {
                Self::add_face(self.vertices.as_mut().unwrap(), self.indices.as_mut().unwrap(), *pos, IVec3::new(0, -1, 0), &mut index, texture_coords);
            }
            
            //Left Face
            if self.block_is_air(IVec3::new(pos.x - 1, pos.y, pos.z)) {
                Self::add_face(self.vertices.as_mut().unwrap(), self.indices.as_mut().unwrap(), *pos, IVec3::new(-1, 0, 0), &mut index, texture_coords);
            }
            
            //Right Face
            if self.block_is_air(IVec3::new(pos.x + 1, pos.y, pos.z)) {
                Self::add_face(self.vertices.as_mut().unwrap(), self.indices.as_mut().unwrap(), *pos, IVec3::new(1, 0, 0), &mut index, texture_coords);
            }
        }
        
        self.setup_buffers(&gl);
        
        //TODO: Reload neighbor chunk
    }
    
    fn block_is_air(&self, pos: IVec3) -> bool {
        self.get_block(pos) == 0
        
        //TODO: Check neighbor chunk
    }
    
    fn add_face(vertices: &mut Vec<f32>, indices: &mut Vec<i32>, pos: IVec3, normal: IVec3, index: &mut i32, texture_coords: [Vec2; 4]) {
        let pos_float: Vec3 = Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5);
        let mut face_vertices: [Vec3; 4] = [Vec3::ZERO; 4];
        
        match normal {
            
            // Front Face
            IVec3 { x: 0, y: 0, z: 1 } => {
                face_vertices[0] = pos_float + Vec3::new(-0.5, -0.5, 0.5);
                face_vertices[1] = pos_float + Vec3::new(0.5, -0.5, 0.5);
                face_vertices[2] = pos_float + Vec3::new(0.5, 0.5, 0.5);
                face_vertices[3] = pos_float + Vec3::new(-0.5, 0.5, 0.5);
            }
            
            // Back Face
            IVec3 { x: 0, y: 0, z: -1 } => {
                face_vertices[0] = pos_float + Vec3::new(0.5, -0.5, -0.5);
                face_vertices[1] = pos_float + Vec3::new(-0.5, -0.5, -0.5);
                face_vertices[2] = pos_float + Vec3::new(-0.5, 0.5, -0.5);
                face_vertices[3] = pos_float + Vec3::new(0.5, 0.5, -0.5);
            }
            
            // Top Face
            IVec3 { x: 0, y: 1, z: 0 } => {
                face_vertices[0] = pos_float + Vec3::new(-0.5, 0.5, -0.5);
                face_vertices[1] = pos_float + Vec3::new(0.5, 0.5, -0.5);
                face_vertices[2] = pos_float + Vec3::new(0.5, 0.5, 0.5);
                face_vertices[3] = pos_float + Vec3::new(-0.5, 0.5, 0.5);
            }
            
            // Bottom Face
            IVec3 { x: 0, y: -1, z: 0 } => {
                face_vertices[0] = pos_float + Vec3::new(-0.5, -0.5, 0.5);
                face_vertices[1] = pos_float + Vec3::new(0.5, -0.5, 0.5);
                face_vertices[2] = pos_float + Vec3::new(0.5, -0.5, -0.5);
                face_vertices[3] = pos_float + Vec3::new(-0.5, -0.5, -0.5);
            }
            
            // Left Face
            IVec3 { x: -1, y: 0, z: 0 } => {
                face_vertices[0] = pos_float + Vec3::new(-0.5, -0.5, -0.5);
                face_vertices[1] = pos_float + Vec3::new(-0.5, -0.5, 0.5);
                face_vertices[2] = pos_float + Vec3::new(-0.5, 0.5, 0.5);
                face_vertices[3] = pos_float + Vec3::new(-0.5, 0.5, -0.5);
            }
            
            // Right Face
            IVec3 { x: 1, y: 0, z: 0 } => {
                face_vertices[0] = pos_float + Vec3::new(0.5, -0.5, 0.5);
                face_vertices[1] = pos_float + Vec3::new(0.5, -0.5, -0.5);
                face_vertices[2] = pos_float + Vec3::new(0.5, 0.5, -0.5);
                face_vertices[3] = pos_float + Vec3::new(0.5, 0.5, 0.5);
            }
            
            _ => {}
        }
        
        for i in 0..4 {
            vertices.push(face_vertices[i].x);
            vertices.push(face_vertices[i].y);
            vertices.push(face_vertices[i].z);
            vertices.push(texture_coords[i].x);
            vertices.push(texture_coords[i].y);
        }

        indices.push(*index + 0);
        indices.push(*index + 1);
        indices.push(*index + 2);
        indices.push(*index + 2);
        indices.push(*index + 3);
        indices.push(*index + 0);
        *index += 4;
    }
    
    fn setup_buffers(&mut self, gl: &Context) {
        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, self.vertices.as_ref().unwrap().align_to::<u8>().1, glow::STATIC_DRAW);
            
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 5 * size_of::<f32>() as i32, 0);
            gl.enable_vertex_attrib_array(0);
            
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 5 * size_of::<f32>() as i32, 3 * size_of::<f32>() as i32);
            gl.enable_vertex_attrib_array(1);
            
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
    
    pub fn get_block(&self, block_pos: IVec3) -> u8 {
        *self.blocks.get(&block_pos).unwrap_or(&0)
    }
    
    pub fn set_block(&mut self, block_pos: IVec3, id: u8) {
        if id == 0 {
            self.blocks.remove(&block_pos);
        } else {
            self.blocks.insert(block_pos, id);
        }
    }
    
    pub fn initialize(&mut self) {
        self.initialize_blocks();
    }
}
