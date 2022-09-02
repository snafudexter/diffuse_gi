use log::info;

use crate::model::Model;

#[derive(Copy, Clone)]
struct SVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

implement_vertex!(SVertex, position, tex_coords);

struct Space {
    vertices: glium::VertexBuffer<SVertex>,
    index_buffer: glium::IndexBuffer<u16>,
    program: glium::Program,
}

pub struct SkyRenderSystem {
    space: Space,
    model: Model,
    program: glium::Program,
    indices: glium::index::NoIndices,
    scene_draw_params: glium::DrawParameters<'static>,
    m_fWavelength: [f32; 3],
    m_fWavelength4: [f32; 3],
    m_fInnerRadius: f32,
    m_fOuterRadius: f32,
    m_fScale: f32,
    m_Kr: f32,
    m_Kr4PI: f32,
    m_Km: f32,
    m_Km4PI: f32,
    m_ESun: f32,
    m_g: f32,
    m_fRayleighScaleDepth: f32,
}

impl SkyRenderSystem {
    pub fn new(display: &glium::Display) -> Self {
        let vertices = glium::VertexBuffer::new(
            display,
            &[
                SVertex {
                    position: [-4.0f32, 4.0f32, -15.0f32],
                    tex_coords: [0f32, 0f32],
                },
                SVertex {
                    position: [-4.0f32, -4.0f32, -15.0f32],
                    tex_coords: [0f32, 1f32],
                },
                SVertex {
                    position: [4.0f32, -4.0f32, -15.0f32],
                    tex_coords: [1f32, 1f32],
                },
                SVertex {
                    position: [4.0f32, 4.0f32, -15.0f32],
                    tex_coords: [1f32, 0f32],
                },
            ],
        )
        .unwrap();

        let index_buffer = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TriangleFan,
            &[3, 2, 1, 0],
        )
        .unwrap();

        let space_vertex_shader_src =
            std::fs::read_to_string("./sky_from_atmoshphere.vert").unwrap();

        let space_fragment_shader_src =
            std::fs::read_to_string("./sky_from_atmoshphere.frag").unwrap();

        let space: Space = Space {
            vertices,
            index_buffer,
            program: glium::Program::from_source(
                display,
                &space_vertex_shader_src,
                &space_fragment_shader_src,
                None,
            )
            .unwrap(),
        };

        let mut model = Model::new("./sphere.obj", display);

        model.set_scale(10.0f32);

        let vertex_shader_src = std::fs::read_to_string("./sky_from_atmoshphere.vert").unwrap();

        let fragment_shader_src = std::fs::read_to_string("./sky_from_atmoshphere.frag").unwrap();

        println!("compiling sky shaders");

        let program =
            glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None)
                .unwrap();
        println!("finished compiling sky shaders");

        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        let scene_draw_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            blend: glium::Blend {
                color: glium::BlendingFunction::Addition {
                    source: glium::LinearBlendingFactor::One,
                    destination: glium::LinearBlendingFactor::One,
                },
                alpha: glium::BlendingFunction::Addition {
                    source: glium::LinearBlendingFactor::One,
                    destination: glium::LinearBlendingFactor::One,
                },
                constant_value: (1.0, 1.0, 1.0, 1.0),
            },
            backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        let mut m_fWavelength: [f32; 3] = [0f32; 3];
        let mut m_fWavelength4: [f32; 3] = [0f32; 3];

        m_fWavelength[0] = 0.650f32; // 650 nm for red
        m_fWavelength[1] = 0.570f32; // 570 nm for green
        m_fWavelength[2] = 0.475f32; // 475 nm for blue
        m_fWavelength4[0] = m_fWavelength[0].powf(4.0f32);
        m_fWavelength4[1] = m_fWavelength[1].powf(4.0f32);
        m_fWavelength4[2] = m_fWavelength[2].powf(4.0f32);

        let m_fInnerRadius = 1.0f32;
        let m_fOuterRadius = 10.25f32;
        let m_fScale = 1f32 / (m_fOuterRadius - m_fInnerRadius);

        let m_Kr = 0.0025f32; // Rayleigh scattering constant
        let m_Kr4PI = m_Kr * 4.0f32 * std::f32::consts::PI;
        let m_Km = 0.0010f32; // Mie scattering constant
        let m_Km4PI = m_Km * 4.0f32 * std::f32::consts::PI;
        let m_ESun = 20.0f32; // Sun brightness constant
        let m_g = -0.990f32; // The Mie phase asymmetry factor
        let m_fRayleighScaleDepth = 0.25f32;

        Self {
            space,
            model,
            program,
            indices,
            scene_draw_params,
            m_fWavelength,
            m_fWavelength4,
            m_fInnerRadius,
            m_fOuterRadius,
            m_fScale,
            m_Kr,
            m_Kr4PI,
            m_Km,
            m_Km4PI,
            m_ESun,
            m_g,
            m_fRayleighScaleDepth,
        }
    }

    pub fn draw(
        &self,
        target: &mut glium::Frame,
        light_position: &[f32; 3],
        view_proj: &[[f32; 4]; 4],
        camera_pos: &[f32; 3],
        camera_height: f32,
    ) {
        use glium::Surface;

        let model: [[f32; 4]; 4] = (cgmath::Matrix4::from_translation((0.0, 0.0, 0.0).into())
            * cgmath::Matrix4::from_scale(10.0))
        .into();

        let scene_draw_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            // blend: glium::Blend {
            //     color: glium::BlendingFunction::Addition {
            //         source: glium::LinearBlendingFactor::One,
            //         destination: glium::LinearBlendingFactor::One,
            //     },
            //     alpha: glium::BlendingFunction::Addition {
            //         source: glium::LinearBlendingFactor::One,
            //         destination: glium::LinearBlendingFactor::One,
            //     },
            //     constant_value: (1.0, 1.0, 1.0, 1.0),
            // },
            // backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };


        let uniforms = &uniform! {
            model: model,
            lightColor: [1f32, 0.9f32, 0.66f32],
            ambientIntensity: 0.3f32,
            v3LightPos: *light_position,
            view_proj: *view_proj,
            v3CameraPos: *camera_pos,
            v3InvWavelength: [1f32/self.m_fWavelength4[0], 1f32/self.m_fWavelength4[1], 1f32/self.m_fWavelength4[2]],
            fCameraHeight: camera_height,
            fCameraHeight2: camera_height * camera_height,
            fOuterRadius2: self.m_fOuterRadius * self.m_fOuterRadius,
            fInnerRadius: self.m_fInnerRadius,
            fKrESun: self.m_Kr*self.m_ESun,
            fKmESun:self.m_Km*self.m_ESun,
            fKr4PI: self.m_Kr4PI,
            fKm4PI: self.m_Km4PI,
            fScale: 1.0f32 / (self.m_fOuterRadius - self.m_fInnerRadius),
            fScaleDepth: self.m_fRayleighScaleDepth,
            fScaleOverScaleDepth:(1.0f32 / (self.m_fOuterRadius - self.m_fInnerRadius)) / self.m_fRayleighScaleDepth,
            g: self.m_g,
            g2: self.m_g * self.m_g
        };

        target
            .draw(
                &self.space.vertices,
                &self.space.index_buffer,
                &self.space.program,
                uniforms,
                &scene_draw_params,
            )
            .unwrap();

        for mesh_object in self.model.get_mesh_objects() {
            let uniforms = &uniform! {
                model: self.model.get_transform(),
                lightColor: [1f32, 0.9f32, 0.66f32],
                ambientIntensity: 0.3f32,
                v3LightPos: *light_position,
                view_proj: *view_proj,
                v3CameraPos: *camera_pos,
                v3InvWavelength: [1f32/self.m_fWavelength4[0], 1f32/self.m_fWavelength4[1], 1f32/self.m_fWavelength4[2]],
                fCameraHeight: camera_height,
                fInnerRadius: self.m_fInnerRadius,
                fKrESun: self.m_Kr*self.m_ESun,
                fKmESun:self.m_Km*self.m_ESun,
                fKr4PI: self.m_Kr4PI,
                fKm4PI: self.m_Km4PI,
                fScale: 1.0f32 / (self.m_fOuterRadius - self.m_fInnerRadius),
                fScaleDepth: self.m_fRayleighScaleDepth,
                fScaleOverScaleDepth:(1.0f32 / (self.m_fOuterRadius - self.m_fInnerRadius)) / self.m_fRayleighScaleDepth,
                g: self.m_g,
                g2: self.m_g * self.m_g
            };
            target
                .draw(
                    mesh_object.get_vertices(),
                    &self.indices,
                    &self.program,
                    uniforms,
                    &self.scene_draw_params,
                )
                .unwrap();
        }
    }
}
