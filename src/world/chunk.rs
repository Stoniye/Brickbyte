use std::collections::VecDeque;
use glam::{IVec2, IVec3, Mat4, Vec2, Vec3};
use glow::{Context, HasContext, NativeBuffer, NativeTexture, NativeVertexArray, Program};
use rand::Rng;

pub const CHUNK_DIMENSION: u8 = 16;
pub const CHUNK_HEIGHT: u8 = 208;

pub struct Chunk {
    blocks: Vec<u8>,
    light_map: Vec<u8>,
    position: IVec2,
    shader: Program,
    vertices: Option<Vec<f32>>,
    indices: Option<Vec<i32>>,
    vertex_array_object: Option<NativeVertexArray>,
    vertex_buffer_object: Option<NativeBuffer>,
    element_buffer_object: Option<NativeBuffer>
}

impl Chunk {
    pub fn new(position: IVec2, shader: Program, noise_map: Vec<Vec<f64>>) -> Self {
        let mut chunk: Chunk = Chunk{
            blocks: vec![0; (CHUNK_DIMENSION as usize) * (CHUNK_HEIGHT as usize) * (CHUNK_DIMENSION as usize)],
            light_map: vec![0; (CHUNK_DIMENSION as usize) * (CHUNK_HEIGHT as usize) * (CHUNK_DIMENSION as usize)],
            position,
            shader,
            vertices: None,
            indices: None,
            vertex_array_object: None,
            vertex_buffer_object: None,
            element_buffer_object: None
        };
        chunk.initialize_blocks(noise_map);
        chunk.calculate_lighting();
        
        chunk
    }
    
    fn initialize_blocks(&mut self, noise_map: Vec<Vec<f64>>) {
        for x in 0..CHUNK_DIMENSION {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_DIMENSION {
                    let y_map: f64 = 50.0 + (noise_map[z as usize][x as usize] * 100.0); // 50 <= y_map <= 150
                    let stone_y: f64 = rand::rng().random_range(3..6) as f64;

                    if y as f64 <= (y_map - stone_y) {
                        self.set_block(IVec3::new(x as i32, y as i32, z as i32), 3);
                    } else if y as f64 <= (y_map - 1.0) {
                        self.set_block(IVec3::new(x as i32, y as i32, z as i32), 2);
                    } else if y as f64 <= y_map {
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
        
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<i32> = Vec::new();

        let mut index: i32 = 0;

        for y in 0..CHUNK_HEIGHT as i32 {
            for z in 0..CHUNK_DIMENSION as i32 {
                for x in 0..CHUNK_DIMENSION as i32 {

                    let block_pos = IVec3::new(x, y, z);
                    let block_type = self.get_block(IVec3::new(x, y, z));

                    // Skip air
                    if block_type == 0 { continue; }

                    let texture_coords = Self::get_texture_coords(&block_type);

                    // Front face (Z + 1)
                    if self.block_is_air(IVec3::new(x, y, z + 1)) {
                        self.add_face(&mut vertices, &mut indices, block_pos, IVec3::new(0, 0, 1), &mut index, texture_coords);
                    }

                    // Back Face (Z - 1)
                    if self.block_is_air(IVec3::new(x, y, z - 1)) {
                        self.add_face(&mut vertices, &mut indices, block_pos, IVec3::new(0, 0, -1), &mut index, texture_coords);
                    }

                    // Top Face (Y + 1)
                    if self.block_is_air(IVec3::new(x, y + 1, z)) {
                        self.add_face(&mut vertices, &mut indices, block_pos, IVec3::new(0, 1, 0), &mut index, texture_coords);
                    }

                    // Bottom Face (Y - 1)
                    if self.block_is_air(IVec3::new(x, y - 1, z)) {
                        self.add_face(&mut vertices, &mut indices, block_pos, IVec3::new(0, -1, 0), &mut index, texture_coords);
                    }

                    // Left Face (X - 1)
                    if self.block_is_air(IVec3::new(x - 1, y, z)) {
                        self.add_face(&mut vertices, &mut indices, block_pos, IVec3::new(-1, 0, 0), &mut index, texture_coords);
                    }

                    // Right Face (X + 1)
                    if self.block_is_air(IVec3::new(x + 1, y, z)) {
                        self.add_face(&mut vertices, &mut indices, block_pos, IVec3::new(1, 0, 0), &mut index, texture_coords);
                    }
                }
            }
        }

        self.vertices = Some(vertices);
        self.indices = Some(indices);
        
        self.setup_buffers(&gl);
    }
    
    fn block_is_air(&self, pos: IVec3) -> bool {
        self.get_block(pos) == 0
    }

    pub fn calculate_lighting(&mut self) {
        let mut queue: VecDeque<IVec3> = VecDeque::new();
        self.light_map.fill(0);

        for x in 0..CHUNK_DIMENSION as i32 {
            for z in 0..CHUNK_DIMENSION as i32 {
                for y in (0..CHUNK_HEIGHT as i32).rev() {
                    let pos = IVec3::new(x, y, z);
                    if !self.block_is_air(pos) {
                        break;
                    }
                    self.set_light(pos, 15);
                    queue.push_back(pos);
                }
            }
        }

        while let Some(pos) = queue.pop_front() {
            let current_light = self.get_light(pos);
            if current_light <= 1 { continue; }

            let neighbors = [
                IVec3::new(1, 0, 0), IVec3::new(-1, 0, 0),
                IVec3::new(0, 1, 0), IVec3::new(0, -1, 0),
                IVec3::new(0, 0, 1), IVec3::new(0, 0, -1),
            ];

            for offset in neighbors {
                let neighbor_pos = pos + offset;

                if self.block_is_air(neighbor_pos) {
                    let neighbor_light = self.get_light(neighbor_pos);

                    if neighbor_light < current_light - 1 {
                        self.set_light(neighbor_pos, current_light - 1);
                        queue.push_back(neighbor_pos);
                    }
                }
            }
        }
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

    fn get_light(&self, pos: IVec3) -> u8 {
        if pos.x < 0 || pos.x >= CHUNK_DIMENSION as i32 || pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 || pos.z < 0 || pos.z >= CHUNK_DIMENSION as i32 {
            return 15;
        }
        self.light_map[Self::get_block_index(pos)]
    }

    fn set_light(&mut self, pos: IVec3, level: u8) {
        if pos.x >= 0 && pos.x < CHUNK_DIMENSION as i32 && pos.y >= 0 && pos.y < CHUNK_HEIGHT as i32 && pos.z >= 0 && pos.z < CHUNK_DIMENSION as i32 {
            self.light_map[Self::get_block_index(pos)] = level;
        }
    }
    
    fn add_face(&self, vertices: &mut Vec<f32>, indices: &mut Vec<i32>, pos: IVec3, normal: IVec3, index: &mut i32, texture_coords: [Vec2; 4]) {
        let pos_float: Vec3 = Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5);
        let mut face_vertices: [Vec3; 4] = [Vec3::ZERO; 4];
        let mut face_light: [f32; 4] = [1.0; 4];
        let light_level = self.get_light(pos + normal);
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
                face_light[0] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(-1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,-1,0)), !self.block_is_air(adjacent + IVec3::new(-1,-1,0)));
                face_light[1] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,-1,0)), !self.block_is_air(adjacent + IVec3::new(1,-1,0)));
                face_light[2] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,1,0)), !self.block_is_air(adjacent + IVec3::new(1,1,0)));
                face_light[3] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(-1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,1,0)), !self.block_is_air(adjacent + IVec3::new(-1,1,0)));
            }
            
            // Back Face
            IVec3 { x: 0, y: 0, z: -1 } => {
                face_vertices[0] = pos_float + Vec3::new(0.5, -0.5, -0.5);
                face_vertices[1] = pos_float + Vec3::new(-0.5, -0.5, -0.5);
                face_vertices[2] = pos_float + Vec3::new(-0.5, 0.5, -0.5);
                face_vertices[3] = pos_float + Vec3::new(0.5, 0.5, -0.5);

                let adjacent = pos + normal;
                face_light[0] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,-1,0)), !self.block_is_air(adjacent + IVec3::new(1,-1,0)));
                face_light[1] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(-1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,-1,0)), !self.block_is_air(adjacent + IVec3::new(-1,-1,0)));
                face_light[2] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(-1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,1,0)), !self.block_is_air(adjacent + IVec3::new(-1,1,0)));
                face_light[3] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,1,0)), !self.block_is_air(adjacent + IVec3::new(1,1,0)));
            }
            
            // Top Face
            IVec3 { x: 0, y: 1, z: 0 } => {
                face_vertices[0] = pos_float + Vec3::new(-0.5, 0.5, -0.5);
                face_vertices[1] = pos_float + Vec3::new(0.5, 0.5, -0.5);
                face_vertices[2] = pos_float + Vec3::new(0.5, 0.5, 0.5);
                face_vertices[3] = pos_float + Vec3::new(-0.5, 0.5, 0.5);

                let adjacent = pos + normal;
                face_light[0] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(-1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,0,-1)), !self.block_is_air(adjacent + IVec3::new(-1,0,-1)));
                face_light[1] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,0,-1)), !self.block_is_air(adjacent + IVec3::new(1,0,-1)));
                face_light[2] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,0,1)), !self.block_is_air(adjacent + IVec3::new(1,0,1)));
                face_light[3] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(-1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,0,1)), !self.block_is_air(adjacent + IVec3::new(-1,0,1)));
            }
            
            // Bottom Face
            IVec3 { x: 0, y: -1, z: 0 } => {
                face_vertices[0] = pos_float + Vec3::new(-0.5, -0.5, 0.5);
                face_vertices[1] = pos_float + Vec3::new(0.5, -0.5, 0.5);
                face_vertices[2] = pos_float + Vec3::new(0.5, -0.5, -0.5);
                face_vertices[3] = pos_float + Vec3::new(-0.5, -0.5, -0.5);

                let adjacent = pos + normal;
                face_light[0] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(-1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,0,1)), !self.block_is_air(adjacent + IVec3::new(-1,0,1)));
                face_light[1] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,0,1)), !self.block_is_air(adjacent + IVec3::new(1,0,1)));
                face_light[2] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,0,-1)), !self.block_is_air(adjacent + IVec3::new(1,0,-1)));
                face_light[3] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(-1,0,0)), !self.block_is_air(adjacent + IVec3::new(0,0,-1)), !self.block_is_air(adjacent + IVec3::new(-1,0,-1)));
            }
            
            // Left Face
            IVec3 { x: -1, y: 0, z: 0 } => {
                face_vertices[0] = pos_float + Vec3::new(-0.5, -0.5, -0.5);
                face_vertices[1] = pos_float + Vec3::new(-0.5, -0.5, 0.5);
                face_vertices[2] = pos_float + Vec3::new(-0.5, 0.5, 0.5);
                face_vertices[3] = pos_float + Vec3::new(-0.5, 0.5, -0.5);
                
                let adjacent = pos + normal;
                face_light[0] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(0,-1,0)), !self.block_is_air(adjacent + IVec3::new(0,0,-1)), !self.block_is_air(adjacent + IVec3::new(0,-1,-1)));
                face_light[1] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(0,-1,0)), !self.block_is_air(adjacent + IVec3::new(0,0,1)), !self.block_is_air(adjacent + IVec3::new(0,-1,1)));
                face_light[2] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(0,1,0)), !self.block_is_air(adjacent + IVec3::new(0,0,1)), !self.block_is_air(adjacent + IVec3::new(0,1,1)));
                face_light[3] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(0,1,0)), !self.block_is_air(adjacent + IVec3::new(0,0,-1)), !self.block_is_air(adjacent + IVec3::new(0,1,-1)));
            }
            
            // Right Face
            IVec3 { x: 1, y: 0, z: 0 } => {
                face_vertices[0] = pos_float + Vec3::new(0.5, -0.5, 0.5);
                face_vertices[1] = pos_float + Vec3::new(0.5, -0.5, -0.5);
                face_vertices[2] = pos_float + Vec3::new(0.5, 0.5, -0.5);
                face_vertices[3] = pos_float + Vec3::new(0.5, 0.5, 0.5);

                let adjacent = pos + normal;
                face_light[0] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(0,-1,0)), !self.block_is_air(adjacent + IVec3::new(0,0,1)), !self.block_is_air(adjacent + IVec3::new(0,-1,1)));
                face_light[1] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(0,-1,0)), !self.block_is_air(adjacent + IVec3::new(0,0,-1)), !self.block_is_air(adjacent + IVec3::new(0,-1,-1)));
                face_light[2] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(0,1,0)), !self.block_is_air(adjacent + IVec3::new(0,0,-1)), !self.block_is_air(adjacent + IVec3::new(0,1,-1)));
                face_light[3] = Self::vertex_ao(!self.block_is_air(adjacent + IVec3::new(0,1,0)), !self.block_is_air(adjacent + IVec3::new(0,0,1)), !self.block_is_air(adjacent + IVec3::new(0,1,1)));
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
    
    fn setup_buffers(&mut self, gl: &Context) {
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
