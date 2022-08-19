use glium::{texture::SrgbTexture2d, GlObject};

pub struct ShadowRenderSystem {
    texture: glium::texture::DepthTexture2d,
    program: glium::program::Program,
    projection_matrix: cgmath::Matrix4<f32>,
    drawable_shadow_texture: std::rc::Rc<SrgbTexture2d>,
}

const SHADOW_SIZE: u32 = 4096 / 2;

impl ShadowRenderSystem {
    pub fn new(display: &glium::Display) -> Self {
        let texture =
            glium::texture::DepthTexture2d::empty(display, SHADOW_SIZE, SHADOW_SIZE).unwrap();

        let vertex_shader_src = std::fs::read_to_string("./shadow.vert").unwrap();

        let fragment_shader_src = std::fs::read_to_string("./shadow.frag").unwrap();

        let program =
            glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None)
                .unwrap();

        let w = 20.0;

        let projection_matrix: cgmath::Matrix4<f32> = cgmath::ortho(-w, w, -w, w, -10.0, 50.0);

        let shadow_texture_to_render = unsafe {
            SrgbTexture2d::from_id(
                display,
                glium::texture::SrgbFormat::U8U8U8U8,
                texture.get_id(),
                false,
                glium::texture::MipmapsOption::NoMipmap,
                glium::texture::Dimensions::Texture2d {
                    width: SHADOW_SIZE,
                    height: SHADOW_SIZE,
                },
            )
        };

        let shadow_texture_to_render = std::rc::Rc::new(shadow_texture_to_render);

        Self {
            texture,
            program,
            projection_matrix,
            drawable_shadow_texture: shadow_texture_to_render,
        }
    }

    pub fn get_shader_program(&self) -> &glium::Program {
        &self.program
    }

    pub fn get_projection_matrix(&self) -> &cgmath::Matrix4<f32> {
        &self.projection_matrix
    }

    pub fn get_view_matrix(&self, light_loc: &cgmath::Point3<f32>) -> cgmath::Matrix4<f32> {
        let view_center: cgmath::Point3<f32> = cgmath::Point3::new(0.0, 0.0, 0.0);
        let view_up: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 1.0, 0.0);
        cgmath::Matrix4::look_at_rh(*light_loc, view_center, view_up)
    }

    pub fn get_shadow_texture(&self) -> &glium::texture::DepthTexture2d {
        &self.texture
    }

    pub fn get_drawable_shadow_texture(&self) -> std::rc::Rc<SrgbTexture2d> {
        self.drawable_shadow_texture.clone()
    }
}
