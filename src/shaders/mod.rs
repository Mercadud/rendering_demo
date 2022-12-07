use vulkano::impl_vertex;
use bytemuck::{Pod, Zeroable};

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
    pub colour: [f32; 3],
    pub object_matrix: [[f32; 4]; 4]
}

impl_vertex!(MonkeInstance, colour, object_matrix);