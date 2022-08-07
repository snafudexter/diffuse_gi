use obj::Obj;

pub struct Model {
    obj: Obj,
}

impl Model {
    pub fn new(path: &str) -> Self {
        let obj = Obj::load(path).unwrap();

        Self { obj }
    }
}
