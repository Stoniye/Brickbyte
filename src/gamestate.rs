// SPDX-FileCopyrightText: © 2025 - 2026 Elias Steininger <elias.st4600@gmail.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::num::NonZeroU32;
use crate::world::player::Player;
use crate::world::world::World;
use egui_winit::State;
use glow::{Context, HasContext, NativeTexture, Program};
use std::sync::Arc;
use egui::{Color32, Stroke, TextureId};
use glam::{IVec2, IVec3, Mat4, Vec3, Vec4};
use glutin::context::PossiblyCurrentContext;
use glutin::surface::{GlSurface, Surface, WindowSurface};
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Window};
use worldgen::noise::perlin::PerlinNoise;
use worldgen::noisemap::{NoiseMap, NoiseMapGenerator, NoiseMapGeneratorBase, Seed, Size, Step};
use crate::world::chunk::CHUNK_DIMENSION;

const POV: f32 = 90.0;

const VERTEX_SHADER: &str = include_str!("shader/vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("shader/fragment.glsl");
const BLOCK_ATLAS: &[u8] = include_bytes!("../res/atlas/block_atlas.raw");
const UI_ATLAS: &[u8] = include_bytes!("../res/atlas/ui_atlas.raw");

#[derive(PartialEq)]
enum Scene {
    Menu,
    Game
}

pub struct GameState {
    world: World,
    gl: Arc<Context>,
    gl_surface: Surface<WindowSurface>,
    gl_context: PossiblyCurrentContext,
    window: Arc<Window>,
    player: Player,
    block_texture: NativeTexture,
    selected_hotbar_slot_index: u8,
    keys_pressed: HashSet<KeyCode>,
    egui_context: egui::Context,
    egui_painter: egui_glow::Painter,
    egui_state: State,
    egui_ui_atlas_id: TextureId,
    egui_block_atlas_id: TextureId,
    program: Option<Program>,
    active_scene: Scene
}

impl GameState {
    pub fn new(gl: Arc<Context>, gl_surface: Surface<WindowSurface>, gl_context: PossiblyCurrentContext, window: Arc<Window>, egui_context: egui::Context, mut egui_painter: egui_glow::Painter, egui_state: State) -> Self {
        let (block_texture, egui_block_atlas_is, egui_ui_atlas_id) = Self::load_textures(&gl, &mut egui_painter);

        let mut gamestate: GameState = Self {
            world: World::new(),
            gl,
            gl_surface,
            gl_context,
            window,
            player: Player::new(),
            block_texture,
            selected_hotbar_slot_index: 0,
            keys_pressed: HashSet::new(),
            egui_context,
            egui_painter,
            egui_state,
            egui_ui_atlas_id,
            egui_block_atlas_id: egui_block_atlas_is,
            program: None,
            active_scene: Scene::Menu
        };

        gamestate.init_shader_and_buffers();
        gamestate.generate_world();

        gamestate
    }

    fn init_shader_and_buffers(&mut self) {
        let vertex_shader_src = VERTEX_SHADER;

        let fragment_shader_src = FRAGMENT_SHADER;

        unsafe {
            let vertex_shader = self.gl.create_shader(glow::VERTEX_SHADER).unwrap();
            self.gl.shader_source(vertex_shader, &vertex_shader_src);
            self.gl.compile_shader(vertex_shader);
            assert!(self.gl.get_shader_compile_status(vertex_shader));

            let fragment_shader = self.gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            self.gl.shader_source(fragment_shader, &fragment_shader_src);
            self.gl.compile_shader(fragment_shader);
            assert!(self.gl.get_shader_compile_status(fragment_shader));

            let program = self.gl.create_program().unwrap();
            self.gl.attach_shader(program, vertex_shader);
            self.gl.attach_shader(program, fragment_shader);
            self.gl.link_program(program);

            self.gl.delete_shader(vertex_shader);
            self.gl.delete_shader(fragment_shader);
            self.program = Some(program);
        }
    }

    pub fn generate_world(&mut self) {
        let nm = NoiseMap::new(PerlinNoise::new()).set_seed(Seed::of("12345678910")).set_size(Size::of(CHUNK_DIMENSION as i64, CHUNK_DIMENSION as i64)).set_step(Step::of(0.0005, 0.0005));

        const X_CHUNKS: i8 = 2;
        const Y_CHUNKS: i8 = 2;

        for x in -X_CHUNKS..X_CHUNKS {
            for y in -X_CHUNKS..Y_CHUNKS {
                self.world.insert_chunk(IVec2::new(x as i32, y as i32), self.program.unwrap(), nm.generate_chunk(x as i64, y as i64));
            }
        }

        self.world.reload_world(&self.gl);
    }

    fn load_textures(gl: &Context, egui_painter: &mut egui_glow::Painter) -> (NativeTexture, TextureId, TextureId) {
        unsafe {

            //Block
            let block_texture: NativeTexture = gl.create_texture().expect("Failed to create texture var");
            gl.bind_texture(glow::TEXTURE_2D, Some(block_texture));

            let block_buffer: Vec<u8> = BLOCK_ATLAS.to_vec();

            let mut block_floats: Vec<f32> = Vec::with_capacity(block_buffer.len());
            for block_pixel in block_buffer.chunks_exact(4) {
                let r = block_pixel[0] as f32 / 255.0;
                let g = block_pixel[1] as f32 / 255.0;
                let b = block_pixel[2] as f32 / 255.0;
                let a = block_pixel[3] as f32 / 255.0;

                block_floats.extend_from_slice(&[
                    r * a,
                    g * a,
                    b * a,
                    a
                ]);
            }

            let block_bytes: &[u8] = {std::slice::from_raw_parts(block_floats.as_ptr() as *const u8, block_floats.len() * size_of::<f32>())};

            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA32F as i32, 256, 256, 0, glow::RGBA, glow::FLOAT, glow::PixelUnpackData::Slice(Some(block_bytes)));

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);

            //UI
            let ui_texture: NativeTexture = gl.create_texture().expect("Failed to create texture var");
            gl.bind_texture(glow::TEXTURE_2D, Some(ui_texture));

            let ui_buffer: Vec<u8> = UI_ATLAS.to_vec();

            let mut ui_floats: Vec<f32> = Vec::with_capacity(ui_buffer.len());
            for ui_pixel in ui_buffer.chunks_exact(4) {
                let r = ui_pixel[0] as f32 / 255.0;
                let g = ui_pixel[1] as f32 / 255.0;
                let b = ui_pixel[2] as f32 / 255.0;
                let a = ui_pixel[3] as f32 / 255.0;

                ui_floats.extend_from_slice(&[
                    r * a,
                    g * a,
                    b * a,
                    a
                ]);
            }

            let ui_bytes: &[u8] = {std::slice::from_raw_parts(ui_floats.as_ptr() as *const u8, ui_floats.len() * size_of::<f32>())};

            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA32F as i32, 256, 256, 0, glow::RGBA, glow::FLOAT, glow::PixelUnpackData::Slice(Some(ui_bytes)));

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);

            return (block_texture, egui_painter.register_native_texture(block_texture), egui_painter.register_native_texture(ui_texture));
        };
    }

    pub fn new_frame(&mut self, delta_time: f32) {
        self.player.update_pos(delta_time, self.keys_pressed.clone(), &self.world)
    }

    pub fn render(&mut self) {
        match self.active_scene {
            Scene::Menu => {
                self.render_menu();
            }

            Scene::Game => {
                self.render_game();
            }
        }
    }

    fn render_menu(&mut self) {
        let raw_input = self.egui_state.take_egui_input(&self.window);

        let full_output = self.egui_context.run(raw_input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 2.0 - 25.0);

                    if ui.add_sized([200.0, 50.0], egui::Button::new("Start")).clicked() {
                        self.window.set_cursor_grab(CursorGrabMode::Confined).expect("Failed to grab cursor");
                        self.window.set_cursor_visible(false);
                        self.active_scene = Scene::Game;
                    }
                });
            });
        });

        self.egui_state.handle_platform_output(&self.window, full_output.platform_output);

        let primitives = self.egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);

        self.egui_painter.paint_and_update_textures([self.window.inner_size().width, self.window.inner_size().height], full_output.pixels_per_point, &primitives, &full_output.textures_delta);

        for (id, image_delta) in full_output.textures_delta.set {
            self.egui_painter.set_texture(id, &image_delta);
        }

        self.gl_surface.swap_buffers(&self.gl_context).expect("Unable to swap buffers");
    }

    fn render_game(&mut self) {
        let projection = Mat4::perspective_rh_gl(POV.to_radians(), self.window.inner_size().width as f32 / self.window.inner_size().height as f32, 0.1, 100.0);
        let view = Mat4::look_at_rh(self.player.get_head_pos(), self.player.get_head_pos() + self.player.get_camera_front(), Vec3::Y);
        let pv = projection * view;

        unsafe {
            self.gl.enable(glow::DEPTH_TEST);
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.disable(glow::BLEND);

            self.gl.clear_color(0.5, 0.7, 0.9, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            self.world.render_world(&self.gl, pv, Some(self.block_texture));
        }

        let raw_input = self.egui_state.take_egui_input(&self.window);
        self.egui_context.begin_pass(raw_input);

        let painter_layer = self.egui_context.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("UI")));
        let center = self.egui_context.content_rect().center();

        //Crosshair
        painter_layer.line_segment([center - egui::vec2(10.0, 0.0), center + egui::vec2(10.0, 0.0)], Stroke::new(2.0, Color32::GRAY));
        painter_layer.line_segment([center - egui::vec2(0.0, 10.0), center + egui::vec2(0.0, 10.0)], Stroke::new(2.0, Color32::GRAY));

        //Hotbar
        const HOTBAR_WIDTH: f32 = 500.0;
        const HOTBAR_HEIGHT: f32 = 70.0;
        const HOTBAR_POS_HEIGHT: f32 = 10.0;
        const HOTBAR_SLOT_WIDTH: f32 = 55.0;
        const LINE_THICKNESS: u8 = 5;

        painter_layer.rect(egui::Rect::from_two_pos(center + egui::vec2(-HOTBAR_WIDTH / 2.0, center.y - HOTBAR_HEIGHT), center + egui::vec2(HOTBAR_WIDTH / 2.0, center.y - HOTBAR_POS_HEIGHT)), 2.0, Color32::from_black_alpha(100), Stroke::new(LINE_THICKNESS, Color32::GRAY), egui::StrokeKind::Inside);

        for i in 0..8 {
            painter_layer.line_segment([center + egui::vec2(-((HOTBAR_WIDTH / 2.0) - (HOTBAR_SLOT_WIDTH + (LINE_THICKNESS/2) as f32)) + (i as f32 * HOTBAR_SLOT_WIDTH), center.y - HOTBAR_HEIGHT), center + egui::vec2(-((HOTBAR_WIDTH / 2.0) - (HOTBAR_SLOT_WIDTH + (LINE_THICKNESS/2) as f32)) + (i as f32 * HOTBAR_SLOT_WIDTH), center.y - HOTBAR_POS_HEIGHT)], Stroke::new(LINE_THICKNESS, Color32::GRAY));
        }

        painter_layer.rect(egui::Rect::from_two_pos(center + egui::vec2(-HOTBAR_WIDTH / 2.0 + (self.selected_hotbar_slot_index as f32 * HOTBAR_SLOT_WIDTH), center.y - HOTBAR_HEIGHT), center + egui::vec2(-((HOTBAR_WIDTH / 2.0) - (HOTBAR_SLOT_WIDTH + LINE_THICKNESS as f32)) + (self.selected_hotbar_slot_index as f32 * HOTBAR_SLOT_WIDTH), center.y - HOTBAR_POS_HEIGHT)), 2.0, Color32::TRANSPARENT, Stroke::new(LINE_THICKNESS, Color32::WHITE), egui::StrokeKind::Inside);

        const TEX_SIZE: f64 = 256.0;
        const PIXEL_MARGIN: f64 = 0.1 / TEX_SIZE; //Margin is needed because otherwise the textures are not cropped correctly

        for i in 0..5 {
            let rect = egui::Rect::from_two_pos(
                center + egui::vec2(-245.0 + (i as f32 * 55.0), center.y - 65.0),
                center + egui::vec2(-195.0 + (i as f32 * 55.0), center.y - 15.0)
            );

            let uv = egui::Rect::from_min_max(
                egui::pos2((i as f32 * 16.0 + 0.1) / TEX_SIZE as f32, 0.0 + PIXEL_MARGIN as f32),
                egui::pos2(((i + 1) as f32 * 16.0 - 0.1) / TEX_SIZE as f32, 0.0625 - PIXEL_MARGIN as f32)
            );

            painter_layer.image(self.egui_block_atlas_id, rect, uv, Color32::WHITE);
        }

        for i in 0..self.player.get_health() {
            let rect = egui::Rect::from_two_pos(
                center + egui::vec2(-255.0 + (i as f32 * 35.0), center.y - 115.0),
                center + egui::vec2(-205.0 + (i as f32 * 35.0), center.y - 65.0)
            );

            let uv = egui::Rect::from_min_max(
                egui::pos2(0.0, 0.0),
                egui::pos2(0.0625, 0.0625)
            );

            painter_layer.image(self.egui_ui_atlas_id, rect, uv, Color32::WHITE);
        }

        let full_output = self.egui_context.end_pass();

        self.egui_state.handle_platform_output(&self.window, full_output.platform_output);

        let primitives = self.egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);

        self.egui_painter.paint_and_update_textures([self.window.inner_size().width, self.window.inner_size().height], full_output.pixels_per_point, &primitives, &full_output.textures_delta);

        for (id, image_delta) in full_output.textures_delta.set {
            self.egui_painter.set_texture(id, &image_delta);
        }

        self.gl_surface.swap_buffers(&self.gl_context).expect("Unable to swap buffers");
    }

    pub fn window_resized(&self, size: PhysicalSize<u32>) {
        self.gl_surface.resize(&self.gl_context, NonZeroU32::new(size.width.max(1)).unwrap(), NonZeroU32::new(size.height.max(1)).unwrap());
        unsafe {self.gl.viewport(0, 0, size.width as i32, size.height as i32);}
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        let _ = self.egui_state.on_window_event(&self.window, &event);
    }

    pub fn focused(&self, focused: bool) {
        if self.active_scene == Scene::Game {
            if focused{
                if let Err(_e) = self.window.set_cursor_grab(CursorGrabMode::Confined) {
                    //Retry because on X11 grabbing cursor is blocked while tabbing back in
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    self.window.set_cursor_grab(CursorGrabMode::Confined).expect("Failed to grab cursor");
                }
                self.window.set_cursor_visible(false);
            }
            else if !focused{
                self.window.set_cursor_grab(CursorGrabMode::None).expect("Failed to release cursor");
                self.window.set_cursor_visible(true);
            }
        }
    }

    pub fn keyboard_input(&mut self, state: ElementState, key_code: KeyCode) {
        match state {
            ElementState::Pressed => {
                self.keys_pressed.insert(key_code);
            }

            ElementState::Released => {
                self.keys_pressed.remove(&key_code);
            }
        }
    }

    pub fn mouse_button_input(&mut self, state: ElementState, button: MouseButton) {
        if state == ElementState::Pressed {
            let size = self.window.inner_size();

            let projection = Mat4::perspective_rh_gl(POV.to_radians(), size.width as f32 / size.height as f32, 0.1, 100.0);
            let view = Mat4::look_at_rh(self.player.get_head_pos(), self.player.get_head_pos() + self.player.get_camera_front(), Vec3::Y);

            let ndc = Vec4::new((2.0 * (size.width as f32 / 2.0)) / size.width as f32 - 1.0, 1.0 - (2.0 * (size.height as f32 / 2.0)) / size.height as f32, -1.0, 1.0);

            let mut eye = projection.inverse() * ndc;
            eye = Vec4::new(eye.x, eye.y, -1.0, 0.0);

            let world_ray = view.inverse() * eye;
            let ray_dir = Vec3::new(world_ray.x, world_ray.y, world_ray.z).normalize();

            let ray_origin = self.player.get_head_pos();

            let hit = {
                self.world.raycast_block(ray_origin, ray_dir, 10.0)
            };

            if let Some(hit) = hit {
                match button {
                    MouseButton::Left => {
                        self.world.set_block(hit.block_pos, 0, &self.gl.clone());
                    }

                    MouseButton::Right => {
                        let block_pos = hit.prev_block_pos;
                        let player_pos: IVec3 = IVec3::new(self.player.get_pos().x.floor() as i32, self.player.get_pos().y.ceil() as i32, self.player.get_pos().z.floor() as i32);
                        let player_head_pos: IVec3 = IVec3::new(self.player.get_head_pos().x.floor() as i32, self.player.get_head_pos().y.floor() as i32, self.player.get_head_pos().z.floor() as i32);

                        if block_pos != player_pos && block_pos != player_head_pos {
                            self.world.set_block(hit.prev_block_pos, self.selected_hotbar_slot_index + 1, &self.gl.clone());
                        }
                    },

                    _ => ()
                }
            }
        }
    }

    pub fn mouse_motion_input(&mut self, _delta: (f64, f64), event: DeviceEvent) {
        if let DeviceEvent::MouseMotion {delta} = event {
            self.player.update_rotation(delta);
        }
    }

    pub fn mouse_wheel_input(&mut self, delta: MouseScrollDelta) {
        let mut delta_index: i8 = 0;

        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                if y >= 1.0 {delta_index = -1;}
                else if y <= -1.0 {delta_index = 1;}
            }
            MouseScrollDelta::PixelDelta(pos) => {
                if pos.y >= 10.0 {delta_index = -1;}
                else if pos.y <= -10.0 {delta_index = 1;}
            }
        }

        if delta_index != 0 {
            let new_index = self.selected_hotbar_slot_index as i8 + delta_index;
            if new_index < 0 {
                self.selected_hotbar_slot_index = 8;
            } else if new_index > 8 {
                self.selected_hotbar_slot_index = 0;
            } else {
                self.selected_hotbar_slot_index = new_index as u8;
            }
        }
    }
}
