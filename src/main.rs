use std::ffi::CString;
use glam::{Mat4, Vec3};
use glutin::config::{ConfigSurfaceTypes, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext};
use glutin::display::{Display, GlDisplay};
use glutin::surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface};
use std::num::NonZeroU32;
use glow::{Buffer, Context, HasContext, Program, VertexArray};
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopBuilder};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::{Window, WindowId};

struct Brickbyte{
    window: Option<Window>,
    gl_display: Option<Display>,
    gl_context: Option<PossiblyCurrentContext>,
    gl_surface: Option<Surface<WindowSurface>>,
    gl: Option<Context>,
    program: Option<Program>,
    vao: Option<VertexArray>,
    vbo: Option<Buffer>
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
            vao: None,
            vbo: None
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

    fn init_shader_and_buffers(&mut self){
        let gl = self.gl.as_ref().unwrap();

        let vertex_shader_src = r#"
            #version 330 core
            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 color;
            uniform mat4 mvp;
            out vec3 frag_color;
            void main() {
                gl_Position = mvp * vec4(position, 1.0);
                frag_color = color;
            }
        "#;

        let fragment_shader_src = r#"
            #version 330 core
            in vec3 frag_color;
            out vec4 out_color;
            void main() {
                out_color = vec4(frag_color, 1.0);
            }
        "#;

        unsafe {
            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vertex_shader, vertex_shader_src);
            gl.compile_shader(vertex_shader);
            assert!(gl.get_shader_compile_status(vertex_shader));

            let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fragment_shader, fragment_shader_src);
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

        let vertices: Vec<f32> = vec![
            // Front face (red)
            -0.5, -0.5, -0.5, 1.0, 0.0, 0.0,
            0.5, -0.5, -0.5, 1.0, 0.0, 0.0,
            0.5,  0.5, -0.5, 1.0, 0.0, 0.0,
            0.5,  0.5, -0.5, 1.0, 0.0, 0.0,
            -0.5,  0.5, -0.5, 1.0, 0.0, 0.0,
            -0.5, -0.5, -0.5, 1.0, 0.0, 0.0,
            // Back face (green)
            -0.5, -0.5,  0.5, 0.0, 1.0, 0.0,
            0.5, -0.5,  0.5, 0.0, 1.0, 0.0,
            0.5,  0.5,  0.5, 0.0, 1.0, 0.0,
            0.5,  0.5,  0.5, 0.0, 1.0, 0.0,
            -0.5,  0.5,  0.5, 0.0, 1.0, 0.0,
            -0.5, -0.5,  0.5, 0.0, 1.0, 0.0,
            // Left face (blue)
            -0.5,  0.5,  0.5, 0.0, 0.0, 1.0,
            -0.5,  0.5, -0.5, 0.0, 0.0, 1.0,
            -0.5, -0.5, -0.5, 0.0, 0.0, 1.0,
            -0.5, -0.5, -0.5, 0.0, 0.0, 1.0,
            -0.5, -0.5,  0.5, 0.0, 0.0, 1.0,
            -0.5,  0.5,  0.5, 0.0, 0.0, 1.0,
            // Right face (yellow)
            0.5,  0.5,  0.5, 1.0, 1.0, 0.0,
            0.5,  0.5, -0.5, 1.0, 1.0, 0.0,
            0.5, -0.5, -0.5, 1.0, 1.0, 0.0,
            0.5, -0.5, -0.5, 1.0, 1.0, 0.0,
            0.5, -0.5,  0.5, 1.0, 1.0, 0.0,
            0.5,  0.5,  0.5, 1.0, 1.0, 0.0,
            // Bottom face (cyan)
            -0.5, -0.5, -0.5, 0.0, 1.0, 1.0,
            0.5, -0.5, -0.5, 0.0, 1.0, 1.0,
            0.5, -0.5,  0.5, 0.0, 1.0, 1.0,
            0.5, -0.5,  0.5, 0.0, 1.0, 1.0,
            -0.5, -0.5,  0.5, 0.0, 1.0, 1.0,
            -0.5, -0.5, -0.5, 0.0, 1.0, 1.0,
            // Top face (magenta)
            -0.5,  0.5, -0.5, 1.0, 0.0, 1.0,
            0.5,  0.5, -0.5, 1.0, 0.0, 1.0,
            0.5,  0.5,  0.5, 1.0, 0.0, 1.0,
            0.5,  0.5,  0.5, 1.0, 0.0, 1.0,
            -0.5,  0.5,  0.5, 1.0, 0.0, 1.0,
            -0.5,  0.5, -0.5, 1.0, 0.0, 1.0,
        ];

        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices.as_slice().align_to::<u8>().1, glow::STATIC_DRAW);

            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 6 * size_of::<f32>() as i32, 0);
            gl.enable_vertex_attrib_array(0);

            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 6 * size_of::<f32>() as i32, 3 * size_of::<f32>() as i32);
            gl.enable_vertex_attrib_array(1);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            self.vao = Some(vao);
            self.vbo = Some(vbo);
        }
    }
}

impl winit::application::ApplicationHandler for Brickbyte {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Brickbyte");
        let window = event_loop.create_window(window_attributes).unwrap();
        self.window = Some(window);

        if let Some(window) = self.window.take(){
            self.init_gl(&window);
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
                if let (Some(surface), Some(context)) = (self.gl_surface.as_ref(), self.gl_context.as_ref()){
                    surface.resize(
                        context,
                        NonZeroU32::new(size.width.max(1)).unwrap(),
                        NonZeroU32::new(size.height.max(1)).unwrap()
                    )
                }
            }

            WindowEvent::RedrawRequested => {
                let gl = self.gl.as_ref().unwrap();
                let window = self.window.as_ref().unwrap();

                let aspect_ratio = window.inner_size().width as f32 / window.inner_size().height as f32;
                let projection = Mat4::perspective_rh_gl(45.0f32.to_radians(), aspect_ratio, 0.1, 100.0);
                let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 3.0), Vec3::ZERO, Vec3::Y);
                let model = Mat4::IDENTITY;
                let mvp = projection * view * model;

                unsafe {
                    gl.clear_color(0.5, 0.7, 0.9, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

                    gl.use_program(self.program);
                    let mvp_loc = gl.get_uniform_location(self.program.unwrap(), "mvp");
                    gl.uniform_matrix_4_f32_slice(mvp_loc.as_ref(), false, mvp.as_ref());

                    gl.bind_vertex_array(self.vao);
                    gl.draw_arrays(glow::TRIANGLES, 0, 36);
                    gl.bind_vertex_array(None);
                }

                self.gl_surface.as_ref().unwrap().swap_buffers(self.gl_context.as_ref().unwrap()).unwrap();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
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
