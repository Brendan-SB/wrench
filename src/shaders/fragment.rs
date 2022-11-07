pub const MAX_LIGHTS: usize = 256;

vulkano_shaders::shader! {
    ty: "fragment",
    path: "src/shaders/fragment.glsl"
}
