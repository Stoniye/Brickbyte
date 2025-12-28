// SPDX-FileCopyrightText: © 2025 - 2026 Elias Steininger <elias.st4600@gmail.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![windows_subsystem = "windows"]
mod world;
mod gamestate;

use crate::gamestate::GameState;
use glow::{Context};
use glutin::config::{ConfigSurfaceTypes, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext};
use glutin::display::{Display, DisplayApiPreference, GlDisplay};
use glutin::surface::{Surface, SurfaceAttributesBuilder, WindowSurface};
use std::ffi::CString;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;
use egui_winit::State;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopBuilder};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::{Window, WindowId};

struct Brickbyte{
    window: Option<Arc<Window>>,
    gamestate: Option<GameState>,
    last_update: Instant
}

impl Brickbyte {
    fn new() -> Self {
        Brickbyte{
            window: None,
            gamestate: None,
            last_update: Instant::now()
        }
    }

    fn init_gl(&mut self, window: &Window) -> (Surface<WindowSurface>, PossiblyCurrentContext, Arc<Context>, egui::Context, egui_glow::Painter, State) {

        //gl
        let raw_display = window.display_handle().unwrap().as_raw();

        #[cfg(target_os = "windows")]
        let preference = DisplayApiPreference::Wgl(None);

        #[cfg(not(target_os = "windows"))]
        let preference = DisplayApiPreference::Egl;

        let gl_display = unsafe {Display::new(raw_display, preference).unwrap()};

        let template = ConfigTemplateBuilder::new().with_surface_type(ConfigSurfaceTypes::WINDOW).with_alpha_size(8).build();
        let configs: Vec<_> = unsafe {gl_display.find_configs(template).unwrap().collect()};
        let config = configs.into_iter().next().expect("No GL config found");

        let context_attributes = ContextAttributesBuilder::new().with_context_api(ContextApi::OpenGl(Some(glutin::context::Version::new(3, 3)))).build(Some(window.window_handle().unwrap().as_raw()));
        let not_current_context = unsafe {gl_display.create_context(&config, &context_attributes).unwrap()};

        let raw_window = window.window_handle().unwrap().as_raw();
        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(raw_window, NonZeroU32::new(window.inner_size().width).unwrap(), NonZeroU32::new(window.inner_size().height).unwrap());
        let gl_surface = unsafe {gl_display.create_window_surface(&config, &attrs).unwrap()};

        let gl_context = not_current_context.make_current(&gl_surface).unwrap();

        let gl = unsafe {Context::from_loader_function(|s| {gl_display.get_proc_address(&CString::new(s).unwrap()) as *const _ })};
        let gl_arc = Arc::new(gl);

        //egui
        let egui_context = egui::Context::default();
        let egui_painter = egui_glow::Painter::new(gl_arc.clone(), "", None, true).expect("Failed to create egui painter");
        let egui_state = State::new(egui_context.clone(), egui::ViewportId::ROOT, &window, None, None, None);

        (gl_surface, gl_context, gl_arc, egui_context, egui_painter, egui_state)
    }
}

impl winit::application::ApplicationHandler for Brickbyte {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window: Arc<Window> = Arc::new(event_loop.create_window(Window::default_attributes().with_title("Brickbyte").with_inner_size(PhysicalSize::new(1200.0, 800.0))).unwrap());

        let (gl_surface, gl_context, gl, egui_context, egui_painter, egui_state) = self.init_gl(&window);

        self.window = Some(window.clone());

        let game = GameState::new(gl, gl_surface, gl_context, window, egui_context, egui_painter, egui_state);

        self.gamestate = Some(game);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {

        self.gamestate.as_mut().unwrap().window_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                self.gamestate.as_ref().unwrap().window_resized(size);
            }

            WindowEvent::KeyboardInput {event: KeyEvent {physical_key: PhysicalKey::Code(key_code), state, ..}, ..} => {
                if state == ElementState::Pressed && key_code == KeyCode::Backspace {
                    event_loop.exit();
                }

                self.gamestate.as_mut().unwrap().keyboard_input(state, key_code);
            }

            WindowEvent::MouseInput {state, button, ..} => {
                self.gamestate.as_mut().unwrap().mouse_button_input(state, button);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                self.gamestate.as_mut().unwrap().mouse_wheel_input(delta);
            }

            WindowEvent::Focused(focused) => {
                self.gamestate.as_ref().unwrap().focused(focused);
            }

            WindowEvent::RedrawRequested => {
                self.gamestate.as_mut().unwrap().render();
            }

            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion {delta} = event {
            self.gamestate.as_mut().unwrap().mouse_motion_input(delta, event);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        self.gamestate.as_mut().unwrap().new_frame(delta_time);

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoopBuilder::default().build().unwrap();
    let mut brickbyte = Brickbyte::new();
    event_loop.run_app(&mut brickbyte).unwrap();
}
