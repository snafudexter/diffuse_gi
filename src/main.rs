#[macro_use]
extern crate glium;

mod camera;
mod teapot;
mod triangle;
mod model;

use pollster::FutureExt;
use triangle::Triangle;
use model::Model;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

struct State {
    display: glium::Display,
    camera: camera::Camera,
    camera_controller: camera::CameraController,
    projection: camera::Projection,
    mouse_pressed: bool,
}

impl State {
    async fn new(event_loop: &glium::glutin::event_loop::EventLoop<()>) -> Self {
        use glium::{glutin, Surface};

        let wb = glutin::window::WindowBuilder::new()
            .with_title("Diffue GI")
            .with_inner_size(glutin::dpi::PhysicalSize {
                width: WIDTH,
                height: HEIGHT,
            });
        let cb = glutin::ContextBuilder::new()
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 6)))
            .with_depth_buffer(24)
            .with_vsync(true)
            .with_double_buffer(Some(true));

        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        let camera = camera::Camera::new((0.0, 1.0, 2.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let camera_controller = camera::CameraController::new(4.0, 0.6);

        let projection = camera::Projection::new(WIDTH, HEIGHT, cgmath::Deg(45.0), 0.1, 100.0);

        Self {
            display,
            camera,
            camera_controller,
            projection,
            mouse_pressed: false,
        }
    }

    pub fn get_display_ref(self: &Self) -> &glium::Display {
        &self.display
    }

    fn update(&mut self, dt: std::time::Duration) {
        // UPDATED!
        self.camera_controller.update_camera(&mut self.camera, dt);
    }

    fn input(self: &mut Self, event: &glium::glutin::event::WindowEvent) -> bool {
        match event {
            glium::glutin::event::WindowEvent::KeyboardInput {
                input:
                    glium::glutin::event::KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            glium::glutin::event::WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            glium::glutin::event::WindowEvent::MouseInput {
                button: glium::glutin::event::MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == glium::glutin::event::ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    fn resize(&mut self, new_size: glium::glutin::dpi::PhysicalSize<u32>) {
        // UPDATED!
        if new_size.width > 0 && new_size.height > 0 {
            self.projection.resize(new_size.width, new_size.height);
            // self.size = new_size;
            // self.config.width = new_size.width;
            // self.config.height = new_size.height;
            // self.surface.configure(&self.device, &self.config);
            // self.depth_texture =
            //     texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }
}

fn main() {
    let app = async {
        let event_loop = glium::glutin::event_loop::EventLoop::new();

        let mut state: State = State::new(&event_loop).await;

        let mut egui_glium = egui_glium::EguiGlium::new(state.get_display_ref());

        let triangle = Triangle::new(state.get_display_ref());
        let sponza = Model::new("./Sponza/sponza.obj");

        let mut last_render_time = std::time::Instant::now();

        event_loop.run(move |event, _, control_flow| {
            let mut redraw = || {
                let mut quit = false;
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                let repaint_after = egui_glium.run(state.get_display_ref(), |egui_ctx| {
                    egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
                        ui.heading(format!("Last render time {:?}", dt.as_micros()));
                        ui.label(format!("FPS {:?}", (1000000.0 / dt.as_micros() as f32)));
                        if ui.button("Quit").clicked() {
                            println!("clicked");
                            quit = true;
                        }
                    });
                });

                *control_flow = if quit {
                    glium::glutin::event_loop::ControlFlow::Exit
                } else if repaint_after {
                    state
                        .get_display_ref()
                        .gl_window()
                        .window()
                        .request_redraw();
                    glium::glutin::event_loop::ControlFlow::Poll
                } else {
                    glium::glutin::event_loop::ControlFlow::Wait
                };

                {
                    state.update(dt);
                    use glium::Surface as _;
                    let mut target = state.get_display_ref().draw();

                    let color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                    target.clear_color_and_depth((color[0], color[1], color[2], color[3]), 1.0);

                    // draw things behind egui here
                    triangle.draw(&mut target, &state.camera, &state.projection);

                    egui_glium.paint(state.get_display_ref(), &mut target);

                    // draw things on top of egui here

                    target.finish().unwrap();
                }
            };

            match event {
                glium::glutin::event::Event::MainEventsCleared => state
                    .get_display_ref()
                    .gl_window()
                    .window()
                    .request_redraw(),
                glium::glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
                glium::glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

                glium::glutin::event::Event::DeviceEvent {
                    event: glium::glutin::event::DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    if state.mouse_pressed {
                        state.camera_controller.process_mouse(delta.0, delta.1)
                    }
                }

                glium::glutin::event::Event::WindowEvent { ref event, .. }
                    if !state.input(event) =>
                {
                    use glium::glutin::event::{
                        ElementState, KeyboardInput, VirtualKeyCode, WindowEvent,
                    };

                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = glium::glutin::event_loop::ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }

                    if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                        *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                    }

                    egui_glium.on_event(&event);

                    state
                        .get_display_ref()
                        .gl_window()
                        .window()
                        .request_redraw(); // TODO(emilk): ask egui if the events warrants a repaint instead
                }
                glium::glutin::event::Event::NewEvents(
                    glium::glutin::event::StartCause::ResumeTimeReached { .. },
                ) => {
                    state
                        .get_display_ref()
                        .gl_window()
                        .window()
                        .request_redraw();
                }
                _ => (),
            }
        });
    };

    app.block_on();
}
