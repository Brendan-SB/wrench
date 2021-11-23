pub struct Shaders {
    pub vertex: Vec<u32>,
    pub fragment: Vec<u32>,
}

impl Shaders {
    pub fn new(vertex: Vec<u32>, fragment: Vec<u32>) -> Self {
        Self { vertex, fragment }
    }
}
