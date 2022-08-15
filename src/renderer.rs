use crate::{
    camera::Camera,
    model::Model,
    model_render_system::{ModelRenderSystem},
};

pub struct Renderer {
    model_render_system: ModelRenderSystem,
}

impl Renderer {
    pub fn new(display: &glium::Display) -> Self {
        let model_render_system = ModelRenderSystem::new(display);

        Self {
            model_render_system,
        }
    }

    pub fn render(&self, display: &glium::Display, camera: &Camera, models: &Vec<Model>) {
        Self::render_scene(self, display, camera, models);
    }

    fn render_scene(&self, display: &glium::Display, camera: &Camera, models: &Vec<Model>) {
        let target = self.model_render_system.begin_frame(display);

        let view_proj: [[f32; 4]; 4] =
            (camera.get_projection_matrix() * camera.get_view_matrix()).into();

        for model in models {
            model.draw(
                &mut target,
                self.model_render_system.get_draw_params(),
                self.model_render_system.get_shader_program(),
                &view_proj
            );
        }

        self.model_render_system.end_frame(&target);
    }
}
