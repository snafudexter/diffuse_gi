use crate::camera::{Camera, Projection};
use crate::teapot;
use glium;

pub struct Triangle {
    vertices: glium::VertexBuffer<teapot::Vertex>,
    normals: glium::VertexBuffer<teapot::Normal>,
    indices: glium::IndexBuffer<u16>,
    program: glium::Program,
}

impl Triangle {
    pub fn new(display: &glium::Display) -> Self {
        let vertices = glium::VertexBuffer::new(display, &teapot::VERTICES).unwrap();
        let normals = glium::VertexBuffer::new(display, &teapot::NORMALS).unwrap();
        let indices = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &teapot::INDICES,
        )
        .unwrap();
        let vertex_shader_src = r#"
        #version 460

        in vec3 position;
        in vec3 normal;

        out vec3 v_normal;

        uniform mat4 view_proj;
        uniform mat4 matrix;
    
        void main() {
            v_normal = transpose(inverse(mat3(matrix))) * normal;
            gl_Position = view_proj * matrix * vec4(position, 1.0);
        }
    "#;

        let fragment_shader_src = r#"
        #version 460 

        in vec3 v_normal;
        out vec4 color;

        uniform vec3 u_light;

        void main() {
            float brightness = dot(normalize(v_normal), normalize(u_light));
            vec3 dark_color = vec3(0.6, 0.0, 0.0);
            vec3 regular_color = vec3(1.0, 0.0, 0.0);
            color = vec4(mix(dark_color, regular_color, brightness), 1.0);
        }
    "#;

        let program =
            glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None)
                .unwrap();

        Self {
            vertices,
            normals,
            indices,
            program,
        }
    }

    pub fn draw(&self, frame: &mut glium::Frame, camera: &Camera, projection: &Projection) {
        use glium::Surface;

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let view_proj: [[f32; 4]; 4] = (projection.calc_matrix() * camera.calc_matrix()).into();

        frame
            .draw(
                (&self.vertices, &self.normals),
                &self.indices,
                &self.program,
                &uniform! {matrix: [
                    [0.01, 0.0, 0.0, 0.0],
                    [0.0, 0.01, 0.0, 0.0],
                    [0.0, 0.0, 0.01, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ], u_light: [-1.0, 0.4, 0.9f32], view_proj: view_proj},
                &params,
            )
            .unwrap();
    }
}
