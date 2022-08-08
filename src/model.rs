use obj::Obj;
#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

#[derive(Copy, Clone)]
pub struct Normal {
    normal: (f32, f32, f32),
}
implement_vertex!(Normal, normal);

struct Object {
    vertices: glium::VertexBuffer<Vertex>,
    normals: glium::VertexBuffer<Normal>,
    indices: glium::IndexBuffer<u16>,
}
pub struct Model {
    // obj: Obj,
    // program: glium::Program,
}

impl Model {
    pub fn new(path: &str) -> Self {
        let obj = Obj::load(path).unwrap();

        let objects: Vec<Object>;

        for object in obj.data.objects {
            for group in object.groups {
                for obj::SimplePolygon(v) in group.polys {
                    for vertex in v {
                        let obj::IndexTuple(v, vt, vn) = vertex;
                        
                    }
                }
            }
        }

        Self {}
    }
}
