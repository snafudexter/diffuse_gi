use glium::Display;
use image::{self, GenericImageView};
use obj::Obj;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
    normal: [f32; 3],
}
implement_vertex!(Vertex, position, tex_coord, normal);

pub struct MeshObject {
    vertices: glium::VertexBuffer<Vertex>,
    diffuse_texture: glium::texture::SrgbTexture2d,
}

impl MeshObject {
    pub fn get_vertices(&self) -> &glium::VertexBuffer<Vertex> {
        &self.vertices
    }

    pub fn get_diffuse_texture(&self) -> &glium::texture::SrgbTexture2d {
        &self.diffuse_texture
    }
}

pub struct Model {
    objects: Vec<MeshObject>,
}

impl Model {
    pub fn new(path: &str, display: &Display) -> Self {
        println!("loading models");
        let mut obj = Obj::load(path).unwrap();
        obj.load_mtls().unwrap();

        let mut materials: Vec<&std::sync::Arc<obj::Material>> = vec![];

        for mtl in obj.data.material_libs.iter() {
            for material in mtl.materials.iter() {
                materials.push(material);
            }
        }

        let mut objects: Vec<MeshObject> = vec![];

        for object in obj.data.objects {
            for group in object.groups {
                let mut positions: Vec<[f32; 3]> = vec![];
                let mut tex_coords: Vec<[f32; 2]> = vec![];
                let mut normals: Vec<[f32; 3]> = vec![];

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

                        positions.push(vertex_position);
                        tex_coords.push(obj.data.texture[vt_index]);
                        normals.push(obj.data.normal[vn_index]); //([0f32, 0f32, 0f32]);
                    }
                }

                // for index in (0..positions.len()).step_by(3) {
                //     let va = positions[index as usize];
                //     let vb = positions[index as usize + 1];
                //     let vc = positions[index as usize + 2];

                //     let a = cgmath::Vector3::new(va[0], va[1], va[2]);
                //     let b = cgmath::Vector3::new(vb[0], vb[1], vb[2]);
                //     let c = cgmath::Vector3::new(vc[0], vc[1], vc[2]);

                //     let edgeAB = b - a;
                //     let edgeAC = c - a;

                //     let areaWeightedNormal = edgeAB.cross(edgeAC);

                //     let normalA = normals[index as usize];
                //     let normalB = normals[index as usize + 1];
                //     let normalC = normals[index as usize + 2];

                //     let mut AN = cgmath::Vector3::new(normalA[0], normalA[1], normalA[2]);
                //     let mut BN = cgmath::Vector3::new(normalB[0], normalB[1], normalB[2]);
                //     let mut CN = cgmath::Vector3::new(normalC[0], normalC[1], normalC[2]);

                //     AN += areaWeightedNormal;
                //     BN += areaWeightedNormal;
                //     CN += areaWeightedNormal;

                //     normals[index as usize] = [AN.x, AN.y, AN.z];
                //     normals[index as usize + 1] = [BN.x, BN.y, BN.z];
                //     normals[index as usize + 2] = [CN.x, CN.y, CN.z];
                // }

                let vertices: Vec<Vertex> = positions
                    .iter()
                    .enumerate()
                    .map(|(i, &position)| Vertex {
                        position: position,
                        tex_coord: tex_coords[i],
                        normal: normals[i],
                    })
                    .collect();

                let mut diffuse_texture =
                    glium::texture::SrgbTexture2d::empty(display, 2, 2).unwrap();
                let base_path = "./Sponza/".to_owned();

                match group.material.unwrap() {
                    obj::ObjMaterial::Ref(_) => todo!(),
                    obj::ObjMaterial::Mtl(material) => {
                        match material.map_kd.as_ref() {
                            Some(diffuse_path) => {
                                let diffuse_path = base_path + diffuse_path;
                                let diffuse_image = image::io::Reader::open(diffuse_path)
                                    .unwrap()
                                    .decode()
                                    .unwrap();
                                let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(
                                    &diffuse_image.to_rgba8(),
                                    diffuse_image.dimensions(),
                                );
                                diffuse_texture =
                                    glium::texture::SrgbTexture2d::new(display, raw_image).unwrap();
                            }
                            None => {}
                        };
                    }
                };

                let object: MeshObject = MeshObject {
                    vertices: glium::VertexBuffer::new(display, &vertices).unwrap(),
                    diffuse_texture,
                };

                objects.push(object);
            }
        }
        println!("finished loading models");
        Self { objects }
    }

    pub fn get_mesh_objects(&self) -> &Vec<MeshObject> {
        &self.objects
    }
}
