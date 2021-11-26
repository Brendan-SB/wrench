#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub position: (f32, f32, f32),
}

vulkano::impl_vertex!(Vertex, position);

#[derive(Default, Copy, Clone)]
pub struct Normal {
    pub normal: (f32, f32, f32),
}

vulkano::impl_vertex!(Normal, normal);
