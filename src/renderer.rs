use cgmath::InnerSpace;
use glium::uniforms::SamplerWrapFunction;

use crate::{
    camera::Camera, model::Model, model_render_system::ModelRenderSystem,
    shadow_render_system::ShadowRenderSystem, sky_render_system::SkyRenderSystem,
};

pub struct Renderer {
    model_render_system: ModelRenderSystem,
    shadow_render_system: ShadowRenderSystem,
    sky_render_system: SkyRenderSystem,
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
            backface_culling: glium::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let shadow_render_system = ShadowRenderSystem::new(display);

        let sky_render_system = SkyRenderSystem::new(display);

        Self {
            model_render_system,
            scene_draw_params,
            shadow_draw_params,
            shadow_render_system,
            sky_render_system,
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
                    model: model.get_transform(),
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

        let light_space_matrix = self.shadow_render_system.get_projection_matrix()
            * self
                .shadow_render_system
                .get_view_matrix(light_position.into());

        let depth_bias_matrix = cgmath::Matrix4::from_scale(0.5)
            * cgmath::Matrix4::from_translation((1.0f32, 1.0f32, 1.0f32).into());

        let light_matrix: [[f32; 4]; 4] = (depth_bias_matrix * light_space_matrix).into();

        let texel_size: [f32; 3] = cgmath::Vector3::new(
            1.0 / crate::shadow_render_system::SHADOW_SIZE as f32,
            1.0 / crate::shadow_render_system::SHADOW_SIZE as f32,
            0.0f32,
        )
        .into();

        let frustum_size = 2.0 * 0.1 * (45.0f32 * 0.5).tan() * camera.get_aspect_ratio();

        //println!("texel size {:?} bias {:?}", texel_size, shadow_bias);

        self.sky_render_system.draw(
            target,
            light_position,
            &view_proj,
            &[
                camera.get_view_position().x,
                camera.get_view_position().y,
                camera.get_view_position().z,
            ],
            cgmath::Vector3::new(
                camera.get_view_position().x,
                camera.get_view_position().y,
                camera.get_view_position().z,
            )
            .magnitude(),
        );

        for model in models {
            for mesh_object in model.get_mesh_objects() {
                let uniforms = &uniform! {
                    model: model.get_transform(),
                    lightColor: [1f32, 0.9f32, 0.66f32],
                    ambientIntensity: 0.3f32,
                    lightPosition: *light_position,
                    view_proj: view_proj,
                    tex: mesh_object.get_diffuse_texture(),
                    shadowMap: shadow_map,
                    light_space_matrix: light_matrix,
                    texelSize: texel_size,
                    frustumSize: frustum_size,
                    distribution: self.shadow_render_system.get_poisson_disk_texture(),
                    ambientColor: *mesh_object.get_ambient_color(),
                    diffuseColor: *mesh_object.get_diffuse_color(),
                    specularColor: *mesh_object.get_specular_color()
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
