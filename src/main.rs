mod world;

use crate::world::chunk::Chunk;
use glam::{IVec2, Mat4, Vec3};
use glow::{Context, HasContext, Program};
use glutin::config::{ConfigSurfaceTypes, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext};
use glutin::display::{Display, GlDisplay};
use glutin::surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface};
use std::collections::HashSet;
use std::ffi::CString;
use std::fs::read_to_string;
use std::num::NonZeroU32;
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopBuilder};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::{CursorGrabMode, Window, WindowId};

struct Brickbyte{
    window: Option<Window>,
    gl_display: Option<Display>,
    gl_context: Option<PossiblyCurrentContext>,
    gl_surface: Option<Surface<WindowSurface>>,
    gl: Option<Context>,
    program: Option<Program>,
    camera_pos: Vec3,
    camera_front: Vec3,
    camera_up: Vec3,
    camera_speed: f32,
    keys_pressed: HashSet<KeyCode>,
    last_update: Instant,
    yaw: f32,
    pitch: f32,
    mouse_sens: f32,
    chunks: Vec<Chunk>
}

impl Brickbyte {
    fn new() -> Self {
        Brickbyte{
            window: None,
            gl_display: None,
            gl_context: None,
            gl_surface: None,
            gl: None,
            program: None,
            camera_pos: Vec3::new(0.0, 0.0, 3.0),
            camera_front: Vec3::new(0.0, 0.0, -1.0),
            camera_up: Vec3::Y,
            camera_speed: 2.5,
            keys_pressed: HashSet::new(),
            last_update: Instant::now(),
            yaw: -90.0,
            pitch: 0.0,
            mouse_sens: 0.04,
            chunks: Vec::new()
        }
    }

    fn init_gl(&mut self, window: &Window) {
        let raw_display = window.display_handle().unwrap().as_raw();
        let gl_display = unsafe {Display::new(raw_display, glutin::display::DisplayApiPreference::Egl).unwrap()};
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
        self.gl = Some(gl);

        unsafe {self.gl.as_ref().unwrap().enable(glow::DEPTH_TEST)};
    }

    fn init_shader_and_buffers(&mut self) {
        let gl = self.gl.as_ref().unwrap();

        let vertex_shader_src = match read_to_string("src/shader/vertex.glsl") {
            Ok(src) => src,
            Err(e) => { panic!("Error reading vertex shader: {}", e); }
        };

        let fragment_shader_src = match read_to_string("src/shader/fragment.glsl") {
            Ok(src) => src,
            Err(e) => { panic!("Error reading fragment shader: {}", e); }
        };

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

        self.chunks.push(Chunk::new(IVec2::new(2, 0), self.program.unwrap()));
        self.chunks.push(Chunk::new(IVec2::new(1, 0), self.program.unwrap()));
        self.chunks.push(Chunk::new(IVec2::new(0, 0), self.program.unwrap()));
        
        for chunk in self.chunks.iter_mut(){
            chunk.reload_chunk(false, gl);
        }
    }

    fn update_camera(&mut self, delta_time: f32) {
        let camera_speed = self.camera_speed * delta_time;
        let camera_right = self.camera_front.cross(self.camera_up).normalize();

        if self.keys_pressed.contains(&KeyCode::KeyW){
            self.camera_pos += camera_speed * self.camera_front;
        }
        if self.keys_pressed.contains(&KeyCode::KeyS){
            self.camera_pos -= camera_speed * self.camera_front;
        }
        if self.keys_pressed.contains(&KeyCode::KeyA){
            self.camera_pos -= camera_speed * camera_right;
        }
        if self.keys_pressed.contains(&KeyCode::KeyD){
            self.camera_pos += camera_speed * camera_right;
        }
        if self.keys_pressed.contains(&KeyCode::ShiftLeft){
            self.camera_pos -= camera_speed * self.camera_up;
        }
        if self.keys_pressed.contains(&KeyCode::Space){
            self.camera_pos += camera_speed * self.camera_up;
        }
    }

    fn update_camera_direction(&mut self) {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        self.camera_front = Vec3::new(yaw_rad.cos() * pitch_rad.cos(), pitch_rad.sin(), yaw_rad.sin() * pitch_rad.cos()).normalize();
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

            WindowEvent::KeyboardInput {event: KeyEvent {physical_key: PhysicalKey::Code(key_code), state, .. }, .. } => {
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

                let aspect_ratio = window.inner_size().width as f32 / window.inner_size().height as f32;
                let projection = Mat4::perspective_rh_gl(90.0f32.to_radians(), aspect_ratio, 0.1, 100.0);
                let view = Mat4::look_at_rh(self.camera_pos, self.camera_pos + self.camera_front, self.camera_up);

                unsafe {
                    gl.clear_color(0.5, 0.7, 0.9, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

                    for chunk in &self.chunks {
                        chunk.render(gl, projection, view);
                    }
                }

                self.gl_surface.as_ref().unwrap().swap_buffers(self.gl_context.as_ref().unwrap()).unwrap();
            }

            _ => {}
        }
    }

    //TODO: Keyboard and mouse input can't be processed at the same time on Wayland
    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion {delta} = event {
            self.yaw += delta.0 as f32 * self.mouse_sens;
            self.pitch -= delta.1 as f32 * self.mouse_sens;
            self.pitch = self.pitch.clamp(-89.0, 89.0);
            self.update_camera_direction();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        self.update_camera(delta_time);

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
