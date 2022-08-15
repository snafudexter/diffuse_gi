use crate::{camera::Camera, model::Model, model_render_system::ModelRenderSystem};

pub struct Renderer {
    model_render_system: ModelRenderSystem,
    draw_params: glium::DrawParameters<'static>,
}

impl Renderer {
    pub fn new(display: &glium::Display) -> Self {
        let model_render_system = ModelRenderSystem::new(display);
        let draw_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            //backface_culling: glium::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };
        Self {
            model_render_system,
            draw_params,
        }
    }

    pub fn render_scene(
        &self,
        display: &glium::Display,
        target: &mut glium::Frame,
        camera: &Camera,
        models: &Vec<Model>,
        light_position: &[f32; 3],
    ) {
        let view_proj: [[f32; 4]; 4] =
            (camera.get_projection_matrix() * camera.get_view_matrix()).into();

        for model in models {
            model.draw(
                display,
                target,
                &self.draw_params,
                self.model_render_system.get_shader_program(),
                &view_proj,
                light_position
            );
        }
    }
}
