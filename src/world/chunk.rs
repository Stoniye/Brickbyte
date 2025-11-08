use glam::{IVec2, IVec3, Mat4, Vec3};
use glow::{Context, HasContext, NativeBuffer, NativeVertexArray, Program, TRIANGLES, UNSIGNED_INT};
use std::collections::HashMap;

const CHUNK_DIMENSION: u8 = 16;
const CHUNK_HEIGHT: u8 = 100;

pub struct Chunk {
    blocks: HashMap<IVec3, u8>,
    position: IVec2,
    shader: Program,
    vertex_buffer_object: Option<NativeBuffer>,
    vertex_array_object: Option<NativeVertexArray>,
    vertices: Option<Vec<f32>>,
    indices: Option<Vec<i32>>
}

impl Chunk {
    pub fn new(position: IVec2, shader: Program) -> Self {
        let mut chunk: Chunk = Chunk{
            blocks: HashMap::new(),
            position,
            shader,
            vertex_buffer_object: None,
            vertex_array_object: None,
            vertices: None,
            indices: None
        };
        chunk.initialize();

        chunk
    }

    fn initialize_blocks(&mut self) {
        for x in 0..CHUNK_DIMENSION {
            for y in 0..CHUNK_DIMENSION {
                for z in 0..CHUNK_HEIGHT {
                    if z <= 16 {
                        self.set_block(IVec3::new(x as i32, y as i32, z as i32), 1);
                    }
                }
            }
        }
    }

    pub fn reload_chunk(&mut self, gl: &Context){
        self.vertices = Some(Vec::new());
        self.indices = Some(Vec::new());

        let mut index: i32 = 0;

        for (pos, block_type) in &self.blocks {
            if *block_type == 0 { continue; }

            //Front face
            if Self::block_is_air(self, IVec3::new(pos.x, pos.y, pos.z + 1)) {
                Self::add_face(self.vertices.as_mut().unwrap(), self.indices.as_mut().unwrap(), *pos, IVec3::new(0, 0, 1), &mut index);
            }

            //Back Face
            if Self::block_is_air(self, IVec3::new(pos.x, pos.y, pos.z - 1)) {
                Self::add_face(self.vertices.as_mut().unwrap(), self.indices.as_mut().unwrap(), *pos, IVec3::new(0, 0, -1), &mut index);
            }

            //Top Face
            if Self::block_is_air(self, IVec3::new(pos.x, pos.y + 1, pos.z)) {
                Self::add_face(self.vertices.as_mut().unwrap(), self.indices.as_mut().unwrap(), *pos, IVec3::new(0, 1, 0), &mut index);
            }

            //Bottom Face
            if Self::block_is_air(self, IVec3::new(pos.x, pos.y - 1, pos.z)) {
                Self::add_face(self.vertices.as_mut().unwrap(), self.indices.as_mut().unwrap(), *pos, IVec3::new(0, -1, 0), &mut index);
            }

            //Left Face
            if Self::block_is_air(self, IVec3::new(pos.x - 1, pos.y, pos.z)) {
                Self::add_face(self.vertices.as_mut().unwrap(), self.indices.as_mut().unwrap(), *pos, IVec3::new(-1, 0, 0), &mut index);
            }

            //Right Face
            if Self::block_is_air(self, IVec3::new(pos.x + 1, pos.y, pos.z)) {
                Self::add_face(self.vertices.as_mut().unwrap(), self.indices.as_mut().unwrap(), *pos, IVec3::new(1, 0, 0), &mut index);
            }
        }

        Self::setup_buffers(self, &gl);
    }

    fn block_is_air(&self, pos: IVec3) -> bool {
        self.get_block(pos) == 0
    }

    fn add_face(vertices: &mut Vec<f32>, indices: &mut Vec<i32>, pos: IVec3, normal: IVec3, index: &mut i32) {
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

            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 3 * size_of::<f32>() as i32, 0);
            gl.enable_vertex_attrib_array(0);

            let ebo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, self.indices.as_ref().unwrap().align_to::<u8>().1, glow::STATIC_DRAW);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

            self.vertex_array_object = Some(vao);
            self.vertex_buffer_object = Some(vbo);
        }
    }

    pub fn render(&self, gl: &Context, pv: Mat4) {
        unsafe {
            let model = Mat4::from_translation(Vec3::new((self.position.x as f32) * CHUNK_DIMENSION as f32, (self.position.y as f32) * CHUNK_DIMENSION as f32, 0.0));
            let mvp = pv * model;

            gl.use_program(Some(self.shader));
            gl.uniform_matrix_4_f32_slice(gl.get_uniform_location(self.shader, "mvp").as_ref(), false, mvp.as_ref());

            gl.bind_vertex_array(self.vertex_array_object);
            gl.draw_elements(TRIANGLES, self.indices.as_ref().unwrap().len() as i32, UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);
        }
    }

    pub fn get_block(&self, block_pos: IVec3) -> u8 {
        self.blocks.get(&block_pos).copied().unwrap_or(0)
    }

    pub fn set_block(&mut self, block_pos: IVec3, id: u8) {
        if id == 0 {
            self.blocks.remove(&block_pos);
        } else {
            self.blocks.insert(block_pos, id);
        }
    }

    pub fn initialize(&mut self) {
        Self::initialize_blocks(self);
    }
}
