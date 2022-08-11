use glium::Display;
use obj::Obj;

use crate::camera::{Camera, Projection};

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
    normal: [f32; 3],
}
implement_vertex!(Vertex, position, tex_coord, normal);

struct MeshObject {
    vertices: glium::VertexBuffer<Vertex>,
    indices: glium::IndexBuffer<u16>,
}

pub struct Model {
    objects: Vec<MeshObject>,
    program: glium::Program,
}

impl Model {
    pub fn new(path: &str, display: &Display) -> Self {
        use std::time::Instant;
        let now = Instant::now();
        let obj = Obj::load(path).unwrap();

        let mut objects: Vec<MeshObject> = vec![];

        for object in obj.data.objects {
            for group in object.groups {
                let mut positions: Vec<[f32; 3]> = vec![];
                let mut tex_coords: Vec<[f32; 2]> = vec![];
                let mut normals: Vec<[f32; 3]> = vec![];
                let mut indices: Vec<u16> = vec![];

                for obj::SimplePolygon(poly) in group.polys {
                    for vertex in poly {
                        let obj::IndexTuple(v, vt, vn) = vertex;
                        let v_index = v;
                        let vt_index = vt.unwrap();
                        let vn_index = vn.unwrap();

                        //(obj.data.normal[vnIndex]);
                        // positions.push(obj.data.position[vIndex]);
                        // indices.push(v as u16);

                        let vertex_position: [f32; 3] = obj.data.position[v_index];

                        let stored_vertex_index_option =
                            positions.iter().position(|&r| r == vertex_position);

                        match stored_vertex_index_option {
                            Some(index) => indices.push(index as u16),
                            None => {
                                positions.push(vertex_position);
                                tex_coords.push(obj.data.texture[vt_index]);
                                normals.push([0f32, 0f32, 0f32]); //(obj.data.normal[vn_index]);
                                indices.push(positions.len() as u16 - 1);
                            }
                        }
                    }
                }

                for index in (0..indices.len()).step_by(3) {
                    let va = positions[indices[index as usize] as usize];
                    let vb = positions[indices[index as usize + 1] as usize];
                    let vc = positions[indices[index as usize + 2] as usize];

                    let A = cgmath::Vector3::new(va[0], va[1], va[2]);
                    let B = cgmath::Vector3::new(vb[0], vb[1], vb[2]);
                    let C = cgmath::Vector3::new(vc[0], vc[1], vc[2]);

                    let edgeAB = B - A;
                    let edgeAC = C - A;

                    let areaWeightedNormal = edgeAB.cross(edgeAC);

                    let normalA = normals[indices[index as usize] as usize];
                    let normalB = normals[indices[index as usize + 1] as usize];
                    let normalC = normals[indices[index as usize + 2] as usize];

                    let mut AN = cgmath::Vector3::new(normalA[0], normalA[1], normalA[2]);
                    let mut BN = cgmath::Vector3::new(normalB[0], normalB[1], normalB[2]);
                    let mut CN = cgmath::Vector3::new(normalC[0], normalC[1], normalC[2]);

                    AN += areaWeightedNormal;
                    BN += areaWeightedNormal;
                    CN += areaWeightedNormal;

                    normals[indices[index as usize] as usize] = [AN.x, AN.y, AN.z];
                    normals[indices[index as usize + 1] as usize] = [BN.x, BN.y, BN.z];
                    normals[indices[index as usize + 2] as usize] = [CN.x, CN.y, CN.z];
                }

                let vertices: Vec<Vertex> = positions
                    .iter()
                    .enumerate()
                    .map(|(i, &position)| Vertex {
                        position: position,
                        tex_coord: tex_coords[i],
                        normal: normals[i],
                    })
                    .collect();

                let object: MeshObject = MeshObject {
                    vertices: glium::VertexBuffer::new(display, &vertices).unwrap(),
                    indices: glium::IndexBuffer::new(
                        display,
                        glium::index::PrimitiveType::TrianglesList,
                        &indices,
                    )
                    .unwrap(),
                };

                objects.push(object);
            }
        }

        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);

        let vertex_shader_src = r#"

            #version 460

            uniform mat4 camera;
            uniform mat4 model;

            in vec3 position;
            in vec2 tex_coord;
            in vec3 normal;

            out vec3 fragVert;
            out vec2 fragTexCoord;
            out vec3 fragNormal;

            void main() {
                // Pass some variables to the fragment shader
                fragTexCoord = tex_coord;
                fragNormal = normal;
                fragVert = position;
                
                // Apply all matrix transformations to vert
                gl_Position = camera * model * vec4(position, 1);
            }
        "#;

        let fragment_shader_src = r#"

            #version 460
        
            uniform mat4 model;
            uniform sampler2D tex;

            uniform vec3 l_position;
            uniform vec3 l_intensities;

            in vec2 fragTexCoord;
            in vec3 fragNormal;
            in vec3 fragVert;

            out vec4 finalColor;

            void main() {
                //calculate normal in world coordinates
                mat3 normalMatrix = transpose(inverse(mat3(model)));
                vec3 normal = normalize(normalMatrix * fragNormal);
                
                //calculate the location of this fragment (pixel) in world coordinates
                vec3 fragPosition = vec3(model * vec4(fragVert, 1));
                
                //calculate the vector from this pixels surface to the light source
                vec3 surfaceToLight = l_position - fragPosition;

                //calculate the cosine of the angle of incidence
                float brightness = dot(normal, surfaceToLight) / (length(surfaceToLight) * length(normal));
                brightness = clamp(brightness, 0, 1);

                //calculate final color of the pixel, based on:
                // 1. The angle of incidence: brightness
                // 2. The color/intensities of the light: light.intensities
                // 3. The texture and texture coord: texture(tex, fragTexCoord)
                // vec4 surfaceColor = texture(tex, fragTexCoord);
                finalColor = vec4(brightness * l_intensities, 1.0); //* surfaceColor.rgb, surfaceColor.a);
            }

        "#;

        let program =
            glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None)
                .unwrap();

        Self { objects, program }
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

        // #[derive(Copy, Clone)]
        // struct Light {
        //     position: [f32; 3],
        //     intensities: [f32; 3],
        // }

        for object in self.objects.iter() {
            frame
                .draw(
                    &object.vertices,
                    &object.indices,
                    &self.program,
                    &uniform! {model: [
                        [0.01, 0.0, 0.0, 0.0],
                        [0.0, 0.01, 0.0, 0.0],
                        [0.0, 0.0, 0.01, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32]
                    ], l_position: [-1.0, 20.4, -3.9f32],
                    l_intensities: [1.0f32, 1.0f32, 1.0f32], camera: view_proj},
                    &params,
                )
                .unwrap();
        }
    }
}
