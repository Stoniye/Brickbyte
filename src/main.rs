#![windows_subsystem = "windows"]
mod world;

use crate::world::player::Player;
use crate::world::world::World;
use egui::{Color32, Stroke};
use egui_winit::State;
use glam::{IVec2, IVec3, Mat4, Vec3, Vec4};
use glow::{Context, HasContext, NativeTexture, Program};
use glutin::config::{ConfigSurfaceTypes, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext};
use glutin::display::{Display, DisplayApiPreference, GlDisplay};
use glutin::surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface};
use std::collections::HashSet;
use std::ffi::CString;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopBuilder};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::{CursorGrabMode, Window, WindowId};

const POV: f32 = 90.0;

const VERTEX_SHADER: &str = include_str!("shader/vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("shader/fragment.glsl");
const BLOCK_ATLAS: &[u8] = include_bytes!("../res/atlas/block_atlas.raw");
const UI_ATLAS: &[u8] = include_bytes!("../res/atlas/ui_atlas.raw");

struct Brickbyte{
    window: Option<Window>,
    gl_display: Option<Display>,
    gl_context: Option<PossiblyCurrentContext>,
    gl_surface: Option<Surface<WindowSurface>>,
    gl: Option<Arc<Context>>,
    egui_state: Option<State>,
    egui_context: Option<egui::Context>,
    egui_painter: Option<egui_glow::Painter>,
    program: Option<Program>,
    keys_pressed: HashSet<KeyCode>,
    last_update: Instant,
    world: World,
    player: Player,
    selected_hotbar_slot_index: u8,
    egui_block_atlas_id: Option<egui::TextureId>,
    egui_ui_atlas_id: Option<egui::TextureId>,
    block_texture: Option<NativeTexture>
}

impl Brickbyte {
    fn new() -> Self {
        Brickbyte{
            window: None,
            gl_display: None,
            gl_context: None,
            gl_surface: None,
            gl: None,
            egui_state: None,
            egui_context: None,
            egui_painter: None,
            program: None,
            keys_pressed: HashSet::new(),
            last_update: Instant::now(),
            world: World::new(),
            player: Player::new(),
            selected_hotbar_slot_index: 0,
            egui_block_atlas_id: None,
            egui_ui_atlas_id: None,
            block_texture: None
        }
    }

    fn init_gl(&mut self, window: &Window) {

        //gl
        let raw_display = window.display_handle().unwrap().as_raw();

        #[cfg(target_os = "windows")]
        let preference = DisplayApiPreference::Wgl(None);

        #[cfg(not(target_os = "windows"))]
        let preference = DisplayApiPreference::Egl;

        let gl_display = unsafe {Display::new(raw_display, preference).unwrap()};
        self.gl_display = Some(gl_display);

        let template = ConfigTemplateBuilder::new().with_surface_type(ConfigSurfaceTypes::WINDOW).with_alpha_size(8).build();
        let configs: Vec<_> = unsafe {self.gl_display.as_ref().unwrap().find_configs(template).unwrap().collect()};
        let config = configs.into_iter().next().expect("No GL config found");

        let context_attributes = ContextAttributesBuilder::new().with_context_api(ContextApi::OpenGl(Some(glutin::context::Version::new(3, 3)))).build(Some(window.window_handle().unwrap().as_raw()));
        let not_current_context = unsafe {self.gl_display.as_ref().unwrap().create_context(&config, &context_attributes).unwrap()};

        let raw_window = window.window_handle().unwrap().as_raw();
        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(raw_window, NonZeroU32::new(window.inner_size().width).unwrap(), NonZeroU32::new(window.inner_size().height).unwrap());
        let gl_surface = unsafe {self.gl_display.as_ref().unwrap().create_window_surface(&config, &attrs).unwrap()};
        self.gl_surface = Some(gl_surface);

        let gl_context = not_current_context.make_current(self.gl_surface.as_ref().unwrap()).unwrap();
        self.gl_context = Some(gl_context);

        let gl = unsafe {Context::from_loader_function(|s| {self.gl_display.as_ref().unwrap().get_proc_address(&CString::new(s).unwrap()) as *const _ })};
        self.gl = Some(Arc::new(gl));

        //egui
        let gl = self.gl.as_ref().unwrap();
        let egui_ctx = egui::Context::default();
        let egui_painter = egui_glow::Painter::new(gl.clone(), "", None, true).expect("Failed to create egui painter");
        let egui_state = State::new(egui_ctx.clone(), egui::ViewportId::ROOT, &window, None, None, None);

        self.egui_state = Some(egui_state);
        self.egui_painter = Some(egui_painter);
        self.egui_context = Some(egui_ctx);


        //Block Atlas
        unsafe {
            let texture: NativeTexture = gl.create_texture().expect("Failed to create texture var");
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            let buffer: Vec<u8> = BLOCK_ATLAS.to_vec();

            let mut floats: Vec<f32> = Vec::with_capacity(buffer.len());
            for pixel in buffer.chunks_exact(4) {
                let r = pixel[0] as f32 / 255.0;
                let g = pixel[1] as f32 / 255.0;
                let b = pixel[2] as f32 / 255.0;
                let a = pixel[3] as f32 / 255.0;

                floats.extend_from_slice(&[
                    r * a,
                    g * a,
                    b * a,
                    a
                ]);
            }

            let bytes: &[u8] = {std::slice::from_raw_parts(floats.as_ptr() as *const u8, floats.len() * size_of::<f32>())};

            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA32F as i32, 256, 256, 0, glow::RGBA, glow::FLOAT, glow::PixelUnpackData::Slice(Some(bytes)));

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);

            self.block_texture = Some(texture);
            self.egui_block_atlas_id = Some(self.egui_painter.as_mut().unwrap().register_native_texture(texture));
        };

        //UI Atlas
        unsafe {
            let texture: NativeTexture = gl.create_texture().expect("Failed to create texture var");
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            let buffer: Vec<u8> = UI_ATLAS.to_vec();

            let mut floats: Vec<f32> = Vec::with_capacity(buffer.len());
            for pixel in buffer.chunks_exact(4) {
                let r = pixel[0] as f32 / 255.0;
                let g = pixel[1] as f32 / 255.0;
                let b = pixel[2] as f32 / 255.0;
                let a = pixel[3] as f32 / 255.0;

                floats.extend_from_slice(&[
                    r * a,
                    g * a,
                    b * a,
                    a
                ]);
            }

            let bytes: &[u8] = {std::slice::from_raw_parts(floats.as_ptr() as *const u8, floats.len() * size_of::<f32>())};

            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA32F as i32, 256, 256, 0, glow::RGBA, glow::FLOAT, glow::PixelUnpackData::Slice(Some(bytes)));

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);

            self.egui_ui_atlas_id = Some(self.egui_painter.as_mut().unwrap().register_native_texture(texture));
        };
    }

    fn init_shader_and_buffers(&mut self) {
        let gl = self.gl.as_ref().unwrap();

        let vertex_shader_src = VERTEX_SHADER;

        let fragment_shader_src = FRAGMENT_SHADER;

        unsafe {
            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vertex_shader, &vertex_shader_src);
            gl.compile_shader(vertex_shader);
            assert!(gl.get_shader_compile_status(vertex_shader));

            let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fragment_shader, &fragment_shader_src);
            gl.compile_shader(fragment_shader);
            assert!(gl.get_shader_compile_status(fragment_shader));

            let program = gl.create_program().unwrap();
            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);

            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);
            self.program = Some(program);
        }

        const X_CHUNKS: u8 = 2;
        const Y_CHUNKS: u8 = 2;

        let gl: &Context = &self.gl.as_mut().unwrap();

        for x in 0..X_CHUNKS {
            for y in 0..Y_CHUNKS {
                self.world.insert_chunk(IVec2::new(x as i32, y as i32), self.program.unwrap());
            }
        }

        self.world.reload_world(gl);
    }
}

impl winit::application::ApplicationHandler for Brickbyte {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Brickbyte").with_inner_size(PhysicalSize::new(1200.0, 800.0));
        let window = event_loop.create_window(window_attributes).unwrap();
        self.window = Some(window);

        if let Some(window) = self.window.take() {
            self.init_gl(&window);
            window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
            window.set_cursor_visible(false);

            self.window = Some(window);
        }

        self.init_shader_and_buffers();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let (Some(surface), Some(context), Some(gl)) = (self.gl_surface.as_ref(), self.gl_context.as_ref(), self.gl.as_ref()) {
                    surface.resize(
                        context,
                        NonZeroU32::new(size.width.max(1)).unwrap(),
                        NonZeroU32::new(size.height.max(1)).unwrap()
                    );
                    unsafe {gl.viewport(0, 0, size.width as i32, size.height as i32);}
                }
            }

            WindowEvent::KeyboardInput {event: KeyEvent {physical_key: PhysicalKey::Code(key_code), state, ..}, ..} => {
                match state {
                    ElementState::Pressed => {
                        self.keys_pressed.insert(key_code);

                        if key_code == KeyCode::Backspace {
                            event_loop.exit();
                        }
                    }
                    ElementState::Released => {
                        self.keys_pressed.remove(&key_code);
                    }
                }
            }

            WindowEvent::MouseInput {state, button, ..} => {
                if state == ElementState::Pressed {
                    let window = self.window.as_ref().unwrap();
                    let size = window.inner_size();

                    let projection = Mat4::perspective_rh_gl(POV.to_radians(), size.width as f32 / size.height as f32, 0.1, 100.0);
                    let view = Mat4::look_at_rh(self.player.get_head_pos(), self.player.get_head_pos() + self.player.get_camera_front(), Vec3::Y);

                    let ndc = Vec4::new((2.0 * (size.width as f32 / 2.0)) / size.width as f32 - 1.0, 1.0 - (2.0 * (size.height as f32 / 2.0)) / size.height as f32, -1.0, 1.0);

                    let mut eye = projection.inverse() * ndc;
                    eye = Vec4::new(eye.x, eye.y, -1.0, 0.0);

                    let world_ray = view.inverse() * eye;
                    let ray_dir = Vec3::new(world_ray.x, world_ray.y, world_ray.z).normalize();

                    let ray_origin = self.player.get_head_pos();

                    if let Some(hit) = self.world.raycast_block(ray_origin, ray_dir, 10.0) {
                        match button {
                            MouseButton::Left => {
                                self.world.set_block(hit.block_pos, 0, self.gl.as_ref().unwrap());
                            }

                            MouseButton::Right => {
                                let block_pos = hit.prev_block_pos;
                                let player_pos: IVec3 = IVec3::new(self.player.get_pos().x.floor() as i32, self.player.get_pos().y.ceil() as i32, self.player.get_pos().z.floor() as i32);
                                let player_head_pos: IVec3 = IVec3::new(self.player.get_head_pos().x.floor() as i32, self.player.get_head_pos().y.floor() as i32, self.player.get_head_pos().z.floor() as i32);

                                if block_pos != player_pos && block_pos != player_head_pos {
                                    self.world.set_block(hit.prev_block_pos, self.selected_hotbar_slot_index + 1, self.gl.as_ref().unwrap());
                                }
                            },

                            _ => ()
                        }
                    }
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let mut delta_index: i8 = 0;

                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        if y >= 1.0 {delta_index = -1;}
                        else if y <= -1.0 {delta_index = 1;}
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        if pos.y >= 1.0 {delta_index = -1;}
                        else if pos.y <= -1.0 {delta_index = 1;}
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

            WindowEvent::Focused(focused) => {
                if let Some(window) = self.window.as_ref() {
                    if focused{
                        if let Err(_e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                            //Retry because on X11 grabbing cursor is blocked while tabbing back in
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
                        }
                        window.set_cursor_visible(false);
                    }
                    else if !focused{
                        window.set_cursor_grab(CursorGrabMode::None).unwrap();
                        window.set_cursor_visible(true);
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                let gl = self.gl.as_ref().unwrap();
                let window = self.window.as_ref().unwrap();
                unsafe {gl.viewport(0, 0, window.inner_size().width as i32, window.inner_size().height as i32);}

                let projection = Mat4::perspective_rh_gl(POV.to_radians(), window.inner_size().width as f32 / window.inner_size().height as f32, 0.1, 100.0);
                let view = Mat4::look_at_rh(self.player.get_head_pos(), self.player.get_head_pos() + self.player.get_camera_front(), Vec3::Y);
                let pv = projection * view;

                unsafe {
                    gl.enable(glow::DEPTH_TEST);
                    gl.disable(glow::SCISSOR_TEST);
                    gl.disable(glow::BLEND);

                    gl.clear_color(0.5, 0.7, 0.9, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

                    self.world.render_world(gl, pv, self.block_texture);
                }

                if let(Some(context), Some(state), Some(painter)) = (self.egui_context.as_ref(), self.egui_state.as_mut(), self.egui_painter.as_mut()) {
                    let raw_input = state.take_egui_input(window);
                    context.begin_pass(raw_input);

                    let painter_layer = context.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("UI")));
                    let center = context.content_rect().center();

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

                        painter_layer.image(self.egui_block_atlas_id.unwrap(), rect, uv, Color32::WHITE);
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

                        painter_layer.image(self.egui_ui_atlas_id.unwrap(), rect, uv, Color32::WHITE);
                    }

                    let full_output = context.end_pass();

                    state.handle_platform_output(window, full_output.platform_output);

                    let primitives = context.tessellate(full_output.shapes, full_output.pixels_per_point);

                    painter.paint_and_update_textures([window.inner_size().width, window.inner_size().height], full_output.pixels_per_point, &primitives, &full_output.textures_delta);

                    for (id, image_delta) in full_output.textures_delta.set {
                        painter.set_texture(id, &image_delta);
                    }
                }

                self.gl_surface.as_ref().unwrap().swap_buffers(self.gl_context.as_ref().unwrap()).unwrap();
            }

            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion {delta} = event {
            self.player.update_rotation(delta);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        self.player.update_pos(delta_time, self.keys_pressed.clone(), &self.world);

        if let Some(window) = &self.window{
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoopBuilder::default().build().unwrap();
    let mut brickbyte = Brickbyte::new();
    event_loop.run_app(&mut brickbyte).unwrap();
}
