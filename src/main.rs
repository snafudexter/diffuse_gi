#[macro_use]
extern crate glium;

mod renderer;
mod camera;
mod model;
mod model_render_system;;

use glium::texture::SrgbTexture2d;
use glium::GlObject;
use model::Model;
use pollster::FutureExt;

use egui_extras::RetainedImage;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

struct State {
    display: glium::Display,
    camera: camera::Camera,
    camera_controller: camera::CameraController,
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

        let camera = camera::Camera::new((-10.0, 7.0, 1.2), cgmath::Deg(-10f32), cgmath::Deg(0.0),WIDTH, HEIGHT, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(2.0, 0.6);

        Self {
            display,
            camera,
            camera_controller,
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
                button: glium::glutin::event::MouseButton::Middle,
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

        let renderer = renderer::Renderer::new(state.get_display_ref());

        let shadow_texture =
            glium::texture::DepthTexture2d::empty(state.get_display_ref(), 1024, 1024).unwrap();
        let shadow_cubemap = glium::texture::DepthCubemap::empty(state.get_display_ref(), 1024).unwrap();

        let shadow_texture_to_render = unsafe {
            SrgbTexture2d::from_id(
                state.get_display_ref(),
                glium::texture::SrgbFormat::U8U8U8U8,
                shadow_texture.get_id(),
                false,
                glium::texture::MipmapsOption::NoMipmap,
                glium::texture::Dimensions::Texture2d {
                    width: 1024,
                    height: 1024,
                },
            )
        };

        let shadow_texture_to_render = std::rc::Rc::new(shadow_texture_to_render);

        let mut egui_glium = egui_glium::EguiGlium::new(state.get_display_ref());

        let sponza = Model::new("./Sponza/sponza.obj", state.get_display_ref());

        let mut last_render_time = std::time::Instant::now();

        let texture_id = egui_glium
            .painter
            .register_native_texture(shadow_texture_to_render);

        let mut start = std::time::Instant::now();
        let mut light_t: f64 = 2.7;

        event_loop.run(move |event, _, control_flow| {
            let mut redraw = || {
                let mut quit = false;
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                let elapsed_dur = start.elapsed();
                let secs =
                    (elapsed_dur.as_secs() as f64) + (elapsed_dur.subsec_nanos() as f64) * 1e-9;
                start = std::time::Instant::now();

                // light_t += secs * 0.5;

                // let light_loc = {
                //     let x = 8.0 * light_t.cos();
                //     let z = 1.0 * light_t.sin();
                //     [x as f32, 5.0, z as f32]
                // };

                let repaint_after = egui_glium.run(state.get_display_ref(), |egui_ctx| {
                    egui::Window::new("Shadow map")
                        .resizable(true)
                        .collapsible(true)
                        .open(&mut false)
                        .show(egui_ctx, |ui| {
                            ui.image(texture_id, egui::vec2(512f32, 512f32));
                        });

                    egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
                        ui.heading(format!("Last render time {:?}", dt.as_micros()));
                        ui.label(format!("FPS {:?}", (1000000.0 / dt.as_micros() as f32)));
                        ui.label(format!("Camera {:?}", &state.camera.position));
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

                    // draw things behind egui here


                    egui_glium.paint(state.get_display_ref(), &mut target);

                    // draw things on top of egui here
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

                glium::glutin::event::Event::WindowEvent { ref event, .. } if !state.input(event) => {
                    
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
