use fast_poisson::Poisson2D;
use glium::{texture::SrgbTexture2d, GlObject};

pub struct ShadowRenderSystem {
    texture: glium::texture::DepthTexture2d,
    program: glium::program::Program,
    projection_matrix: cgmath::Matrix4<f32>,
    drawable_shadow_texture: std::rc::Rc<SrgbTexture2d>,
    poisson_disk: glium::texture::SrgbTexture1d
}

pub const SHADOW_SIZE: u32 = 1024 * 5;

impl ShadowRenderSystem {
    pub fn new(display: &glium::Display) -> Self {
        let texture =
            glium::texture::DepthTexture2d::empty(display, SHADOW_SIZE, SHADOW_SIZE).unwrap();

        let vertex_shader_src = std::fs::read_to_string("./shadow.vert").unwrap();

        let fragment_shader_src = std::fs::read_to_string("./shadow.frag").unwrap();

        println!("compiling shadow shaders");

        let program =
            glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None)
                .unwrap();

        let w = 20.0;

        let projection_matrix: cgmath::Matrix4<f32> = cgmath::ortho(-w, w, -w, w, -10.0, 100.0);

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

        let points: Vec<[f64; 2]> = Poisson2D::new().with_samples(100).generate();

        let points = points
            .iter()
            .map(|point| vec![point[0] as f32, point[1] as f32, 0.0])
            .flatten()
            .collect();

        let distribution = glium::texture::RawImage1d::from_raw_rgb(points);

        let poisson_disk = glium::texture::SrgbTexture1d::new(display, distribution).unwrap();

        let shadow_texture_to_render = std::rc::Rc::new(shadow_texture_to_render);

        println!("{:?}", poisson_disk.get_internal_format().unwrap());

        Self {
            texture,
            program,
            projection_matrix,
            drawable_shadow_texture: shadow_texture_to_render,
            poisson_disk
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

    pub fn get_poisson_disk_texture(&self) -> &glium::texture::SrgbTexture1d {
        &self.poisson_disk
    }
}
