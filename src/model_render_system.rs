use log::info;

pub struct ModelRenderSystem {
    shader_program: glium::Program,
}

impl ModelRenderSystem {
    pub fn new(display: &glium::Display) -> Self {
        let vertex_shader_src = std::fs::read_to_string("./basic.vert").unwrap();

        let fragment_shader_src = std::fs::read_to_string("./basic.frag").unwrap();

        println!("compiling model shaders");
        let program =
            glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None)
                .unwrap();
        println!("finished loading model shaders");
        Self {
            shader_program: program,
        }
    }

    pub fn get_shader_program(&self) -> &glium::Program {
        &self.shader_program
    }
}
