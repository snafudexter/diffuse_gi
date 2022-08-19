use glium::uniforms::SamplerWrapFunction;

use crate::{
    camera::Camera, model::Model, model_render_system::ModelRenderSystem,
    shadow_render_system::ShadowRenderSystem,
};

pub struct Renderer {
    model_render_system: ModelRenderSystem,
    shadow_render_system: ShadowRenderSystem,
    scene_draw_params: glium::DrawParameters<'static>,
    shadow_draw_params: glium::DrawParameters<'static>,
}

impl Renderer {
    pub fn new(display: &glium::Display) -> Self {
        let model_render_system = ModelRenderSystem::new(display);
        let scene_draw_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let shadow_draw_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        let shadow_render_system = ShadowRenderSystem::new(display);

        Self {
            model_render_system,
            scene_draw_params,
            shadow_draw_params,
            shadow_render_system,
        }
    }

    pub fn get_drawable_shadow_texture(&self) -> std::rc::Rc<glium::texture::SrgbTexture2d> {
        self.shadow_render_system.get_drawable_shadow_texture()
    }

    pub fn render_shadows(
        &self,
        display: &glium::Display,
        models: &Vec<Model>,
        light_position: &[f32; 3],
    ) {
        use glium::Surface;

        let mut target = glium::framebuffer::SimpleFrameBuffer::depth_only(
            display,
            self.shadow_render_system.get_shadow_texture(),
        )
        .unwrap();
        target.clear_color(1.0, 1.0, 1.0, 1.0);
        target.clear_depth(1.0);

        let view_proj: [[f32; 4]; 4] = (self.shadow_render_system.get_projection_matrix()
            * self
                .shadow_render_system
                .get_view_matrix(light_position.into()))
        .into();

        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        for model in models {
            for mesh_object in model.get_mesh_objects() {
                let uniforms = &uniform! {
                    model: [
                        [0.01, 0.0, 0.0, 0.0],
                        [0.0, 0.01, 0.0, 0.0],
                        [0.0, 0.0, 0.01, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32]
                    ],
                    view_proj: view_proj,
                };

                target
                    .draw(
                        mesh_object.get_vertices(),
                        &indices,
                        self.shadow_render_system.get_shader_program(),
                        uniforms,
                        &self.shadow_draw_params,
                    )
                    .unwrap();
            }
        }
    }

    pub fn render_scene(
        &self,
        target: &mut glium::Frame,
        camera: &Camera,
        models: &Vec<Model>,
        light_position: &[f32; 3],
    ) {
        use glium::Surface;
        let view_proj: [[f32; 4]; 4] =
            (camera.get_projection_matrix() * camera.get_view_matrix()).into();

        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let shadow_map =
            glium::uniforms::Sampler::new(self.shadow_render_system.get_shadow_texture())
                .wrap_function(SamplerWrapFunction::Clamp)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                .depth_texture_comparison(Some(
                    glium::uniforms::DepthTextureComparison::LessOrEqual,
                ));

        let light_space_matrix: [[f32; 4]; 4] = (self.shadow_render_system.get_projection_matrix()
            * self
                .shadow_render_system
                .get_view_matrix(light_position.into()))
        .into();

        let depth_bias_matrix: [[f32; 4]; 4] = cgmath::Matrix4::new(
            0.5, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.5, 0.5, 1.0,
        )
        .into();

        for model in models {
            for mesh_object in model.get_mesh_objects() {
                let uniforms = &uniform! {
                    model: [
                        [0.01, 0.0, 0.0, 0.0],
                        [0.0, 0.01, 0.0, 0.0],
                        [0.0, 0.0, 0.01, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32]
                    ],
                    LightColor: [1.0f32, 1.0f32, 1.0f32],
                    AmbientIntensity: 0.000f32,
                    LightPosition: *light_position,
                    view_proj: view_proj,
                    tex: mesh_object.get_diffuse_texture(),
                    shadowMap: shadow_map,
                    light_space_matrix: light_space_matrix,
                    depth_bias_matrix: depth_bias_matrix
                };

                target
                    .draw(
                        mesh_object.get_vertices(),
                        &indices,
                        self.model_render_system.get_shader_program(),
                        uniforms,
                        &self.scene_draw_params,
                    )
                    .unwrap();
            }
        }
    }
}
