use crate::camera::Camera;

pub struct ModelRenderSystem {
    shader_program: glium::Program,
    draw_params: glium::DrawParameters<'static>,
}

impl ModelRenderSystem {
    pub fn new(display: &glium::Display) -> Self {
        let vertex_shader_src = std::fs::read_to_string("./basic.vert").unwrap();

        let fragment_shader_src = std::fs::read_to_string("./basic.frag").unwrap();

        let program =
            glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None)
                .unwrap();

        let mut draw_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        Self {
            shader_program: program,
            draw_params,
        }
    }

    pub fn get_draw_params(&self) -> glium::DrawParameters {
        self.draw_params
    }

    pub fn get_shader_program(&self) -> &glium::Program {
        &self.shader_program
    }

    pub fn begin_frame(&self, display: &glium::Display) -> glium::Frame {
        display.draw()
    }

    pub fn end_frame(&self, frame: &glium::Frame) {
        frame.finish().unwrap();
    }
}
