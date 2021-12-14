pub mod assets;
pub mod components;
pub mod engine;
pub mod error;
pub mod scene;
pub mod shaders;

pub use cgmath;
pub use cgmath::*;
pub use vulkano;
pub use vulkano_shaders;
pub use vulkano_win;
pub use winit;

pub mod ecs {
    pub use wecs::*;
    pub use wecs_derive as derive;

    pub mod reexports {
        pub use super::{derive::Component, Component, Entity};
        pub use std::{
            any::Any,
            sync::{Arc, Mutex},
        };
    }
}

pub use engine::Engine;
pub use scene::Scene;
pub use vulkano::image::SampleCount;
