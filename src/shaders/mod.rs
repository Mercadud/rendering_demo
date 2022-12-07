use bytemuck::{Pod, Zeroable};
use vulkano::impl_vertex;

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/shader.vert",
        types_meta: {
            use bytemuck::{Zeroable, Pod};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/shader.frag",
        types_meta: {
            use bytemuck::{Zeroable, Pod};

            #[derive(Clone, Copy, Zeroable, Pod)]
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl_vertex!(Vertex, position, normal);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct MonkeInstance {
    pub transform: [f32; 3],
    pub colour: [f32; 3],
    pub scale: f32,
}

impl_vertex!(MonkeInstance, transform, colour, scale);
