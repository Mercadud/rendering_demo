mod settings;
mod shaders;

use crate::shaders::{fs, vs, MonkeInstance, Vertex};

use math::{perspective_rh, Mat3, Mat4, Vec3};
use obj::{load_obj, Obj};
use std::fs::File;
use std::io::BufReader;
use std::{sync::Arc, time::Instant};
use vulkano::device::Features;
use vulkano::image::SampleCount;

use crate::settings::Levels;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::sync::now;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool, TypedBufferAccess},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, DeviceOwned,
        QueueCreateInfo,
    },
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageAccess, ImageUsage, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{MemoryUsage, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            depth_stencil::DepthStencilState,
            input_assembly::InputAssemblyState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::ShaderModule,
    swapchain::{
        acquire_next_image, AcquireError, Swapchain, SwapchainCreateInfo, SwapchainCreationError,
        SwapchainPresentInfo,
    },
    sync::{FlushError, GpuFuture},
    VulkanLibrary,
};
use vulkano_win::VkSurfaceBuild;
use winit::event::VirtualKeyCode;
use winit::event_loop::ControlFlow;
use winit::window::Fullscreen;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

fn main() {
    let library = VulkanLibrary::new().unwrap();
    let required_extensions = vulkano_win::required_extensions(&library);
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            // Enable enumerating devices that use non-conformant vulkan implementations. (ex. MoltenVK)
            enumerate_portability: true,
            ..Default::default()
        },
    )
    .unwrap();

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .with_title("RENDERING DEMO")
        .with_maximized(true)
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.graphics && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .unwrap();

    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_features: Features {
                fill_mode_non_solid: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();

    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .unwrap();
        let image_format = Some(
            device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0,
        );
        let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage {
                    color_attachment: true,
                    transfer_dst: true,
                    ..ImageUsage::empty()
                },
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .iter()
                    .next()
                    .unwrap(),
                ..Default::default()
            },
        )
        .unwrap()
    };

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let mut vertex_data = vec![];
    let input = BufReader::new(File::open("monke.obj").unwrap());
    let monke: Obj = load_obj(input).unwrap();
    for i in monke.vertices {
        vertex_data.push(Vertex {
            position: i.position,
            normal: i.normal,
        });
    }
    let index_data = monke.indices;

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            vertex_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        vertex_data,
    )
    .unwrap();

    let index_buffer = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            index_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        index_data,
    )
    .unwrap();

    let mut instances = [
        MonkeInstance {
            transform: [0.0, 0.0, -320.0],
            colour: [0.46, 0.15, 0.58],
            scale: 135.0,
        },
        MonkeInstance {
            transform: [0.0, 0.0, -160.0],
            colour: [0.14, 0.71, 0.95],
            scale: 64.0,
        },
        MonkeInstance {
            transform: [0.0, 0.0, -80.0],
            colour: [0.66, 0.31, 0.64],
            scale: 32.0,
        },
        MonkeInstance {
            transform: [0.0, 0.0, -40.0],
            colour: [0.00, 0.81, 0.73],
            scale: 16.0,
        },
        MonkeInstance {
            transform: [0.0, 0.0, -20.0],
            colour: [0.10, 0.89, 0.67],
            scale: 8.0,
        },
        MonkeInstance {
            transform: [0.0, 0.0, -10.0],
            colour: [0.02, 0.71, 0.86],
            scale: 4.0,
        },
        MonkeInstance {
            transform: [0.0, 0.0, -5.0],
            colour: [0.72, 0.04, 0.13],
            scale: 2.0,
        },
        MonkeInstance {
            transform: [0.0, 0.0, 0.0],
            colour: [0.0, 1.0, 0.0],
            scale: 0.4,
        },
    ];

    instances.reverse();

    let instance_buffer = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            vertex_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        instances,
    )
    .unwrap();

    let vs_uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(
        memory_allocator.clone(),
        BufferUsage {
            uniform_buffer: true,
            ..BufferUsage::empty()
        },
        MemoryUsage::Upload,
    );

    let fs_uniform_buffer = CpuBufferPool::<fs::ty::Data>::new(
        memory_allocator.clone(),
        BufferUsage {
            uniform_buffer: true,
            ..BufferUsage::empty()
        },
        MemoryUsage::Upload,
    );

    let vs = vs::load(device.clone()).unwrap();
    let fs = fs::load(device.clone()).unwrap();

    let render_pass_1 = vulkano::single_pass_renderpass!(device.clone(),
        attachments: {
            intermediary: {
                load: Clear,
                store: DontCare,
                format: swapchain.image_format(),
                samples: 8,     // This has to match the image definition.
            },
            color: {
                load: Clear,
                store: Store,
                format: swapchain.image_format(),
                samples: 1,
            },
            depth: {
                load: Clear,
                store: DontCare,
                format: Format::D16_UNORM,
                samples: 8,
            }
        },
        pass: {
            color: [intermediary],
            depth_stencil: {depth},
            resolve: [color]
        }
    )
    .unwrap();

    let render_pass_2 = vulkano::single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.image_format(),
                samples: 1,
            },
            depth: {
                load: Clear,
                store: DontCare,
                format: Format::D16_UNORM,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
    )
    .unwrap();

    let (mut pipeline, mut framebuffers) = window_size_dependent_setup(
        &memory_allocator,
        &vs,
        &fs,
        &images,
        render_pass_2.clone(),
        Levels::ONE,
    );
    let mut recreate_swapchain = false;

    let mut previous_frame_end = Some(now(device.clone()).boxed());
    let rotation_start = Instant::now();

    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let mut level = Levels::ONE;
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(_) => {
                recreate_swapchain = true;
            }
            WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(input) => match input {
                    VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                    VirtualKeyCode::Key1 => {
                        level = Levels::ONE;
                        recreate_swapchain = true;
                    }
                    VirtualKeyCode::Key2 => {
                        level = Levels::TWO;
                        recreate_swapchain = true;
                    }
                    VirtualKeyCode::Key3 => {
                        level = Levels::THREE;
                        recreate_swapchain = true;
                    }
                    VirtualKeyCode::Key4 => {
                        level = Levels::FOUR;
                        recreate_swapchain = true;
                    }
                    VirtualKeyCode::Key5 => {
                        level = Levels::FIVE;
                        recreate_swapchain = true;
                    }
                    _ => {}
                },
                None => {}
            },
            _ => {}
        },
        Event::RedrawEventsCleared => {
            let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
            let dimensions = window.inner_size();
            if dimensions.width == 0 || dimensions.height == 0 {
                return;
            }

            previous_frame_end.as_mut().unwrap().cleanup_finished();

            if recreate_swapchain {
                let (new_swapchain, new_images) = match swapchain.recreate(SwapchainCreateInfo {
                    image_extent: dimensions.into(),
                    ..swapchain.create_info()
                }) {
                    Ok(r) => r,
                    Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                    Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                };

                swapchain = new_swapchain;
                let (new_pipeline, new_framebuffers) = window_size_dependent_setup(
                    &memory_allocator,
                    &vs,
                    &fs,
                    &new_images,
                    {
                        if level >= Levels::FOUR {
                            render_pass_1.clone()
                        } else {
                            render_pass_2.clone()
                        }
                    },
                    level,
                );
                pipeline = new_pipeline;
                framebuffers = new_framebuffers;
                recreate_swapchain = false;
            }

            let vs_uniform_buffer_subbuffer = {
                let rotation = Mat3::from_rotation_y((0) as f32);

                let aspect_ratio =
                    swapchain.image_extent()[0] as f32 / swapchain.image_extent()[1] as f32;
                let proj = perspective_rh(aspect_ratio);
                let view = Mat4::look_at_rh(
                    Vec3::new(0.0, 0.0, 2.0),
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, -1.0, 0.0),
                );
                let scale = Mat4::from_scale(Vec3::new(1.0, 1.0, 1.0));

                let uniform_data = vs::ty::Data {
                    world: Mat4::from_mat3(rotation).to_cols_array_2d(),
                    view: (view * scale).to_cols_array_2d(),
                    proj,
                    lev_2: (level >= Levels::TWO) as u32,
                };

                vs_uniform_buffer.from_data(uniform_data).unwrap()
            };

            let fs_uniform_buffer_subbuffer = {
                let uniform_data = fs::ty::Data {
                    lighting: (level >= Levels::FIVE) as u32,
                };

                fs_uniform_buffer.from_data(uniform_data).unwrap()
            };

            let layout = pipeline.layout().set_layouts().get(0).unwrap();
            let set = PersistentDescriptorSet::new(
                &descriptor_set_allocator,
                layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, vs_uniform_buffer_subbuffer),
                    WriteDescriptorSet::buffer(1, fs_uniform_buffer_subbuffer),
                ],
            )
            .unwrap();

            let (image_index, suboptimal, acquire_future) =
                match acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                };

            if suboptimal {
                recreate_swapchain = true;
            }

            let mut builder = AutoCommandBufferBuilder::primary(
                &command_buffer_allocator,
                queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();
            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: {
                            if level >= Levels::FOUR {
                                vec![
                                    Some([0.0, 0.2, 0.6, 1.0].into()),
                                    Some([0.0, 0.2, 0.6, 1.0].into()),
                                    Some(1f32.into()),
                                ]
                            } else {
                                vec![Some([0.0, 0.2, 0.6, 1.0].into()), Some(1f32.into())]
                            }
                        },
                        ..RenderPassBeginInfo::framebuffer(
                            framebuffers[image_index as usize].clone(),
                        )
                    },
                    SubpassContents::Inline,
                )
                .unwrap()
                .bind_pipeline_graphics(pipeline.clone())
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    pipeline.layout().clone(),
                    0,
                    set,
                )
                .bind_vertex_buffers(0, (vertex_buffer.clone(), instance_buffer.clone()))
                .bind_index_buffer(index_buffer.clone())
                .draw_indexed(
                    index_buffer.len() as u32,
                    instance_buffer.len() as u32,
                    0,
                    0,
                    0,
                )
                .unwrap()
                .end_render_pass()
                .unwrap();
            let command_buffer = builder.build().unwrap();

            let future = previous_frame_end
                .take()
                .unwrap()
                .join(acquire_future)
                .then_execute(queue.clone(), command_buffer)
                .unwrap()
                .then_swapchain_present(
                    queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
                )
                .then_signal_fence_and_flush();

            match future {
                Ok(future) => {
                    previous_frame_end = Some(future.boxed());
                }
                Err(FlushError::OutOfDate) => {
                    recreate_swapchain = true;
                    previous_frame_end = Some(now(device.clone()).boxed());
                }
                Err(e) => {
                    println!("Failed to flush future: {:?}", e);
                    previous_frame_end = Some(now(device.clone()).boxed());
                }
            }
        }
        _ => (),
    });
}

/// This method is called once during initialization, then again whenever the window is resized
fn window_size_dependent_setup(
    memory_allocator: &StandardMemoryAllocator,
    vs: &ShaderModule,
    fs: &ShaderModule,
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
    level: Levels,
) -> (Arc<GraphicsPipeline>, Vec<Arc<Framebuffer>>) {
    let dimensions = images[0].dimensions().width_height();

    let depth_buffer_multi = ImageView::new_default(
        AttachmentImage::transient_multisampled(
            memory_allocator,
            dimensions,
            SampleCount::Sample8,
            Format::D16_UNORM,
        )
        .unwrap(),
    )
    .unwrap();

    let depth_buffer = ImageView::new_default(
        AttachmentImage::transient(memory_allocator, dimensions, Format::D16_UNORM).unwrap(),
    )
    .unwrap();

    let framebuffers = images
        .iter()
        .map(|image| {
            let intermediary = ImageView::new_default(
                AttachmentImage::transient_multisampled(
                    memory_allocator,
                    image.dimensions().width_height(),
                    SampleCount::Sample8,
                    image.format(),
                )
                .unwrap(),
            )
            .unwrap();
            let view = ImageView::new_default(image.clone()).unwrap();
            return if level >= Levels::FOUR {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![intermediary, view, depth_buffer_multi.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            } else {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view, depth_buffer.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            };
        })
        .collect::<Vec<_>>();

    let mut pipeline = GraphicsPipeline::start()
        .vertex_input_state(
            BuffersDefinition::new()
                .vertex::<Vertex>()
                .instance::<MonkeInstance>(),
        )
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([
            Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            },
        ]))
        .fragment_shader(fs.entry_point("main").unwrap(), ());
    if level >= Levels::THREE {
        pipeline = pipeline.depth_stencil_state(DepthStencilState::simple_depth_test());
    } else {
        pipeline = pipeline.depth_stencil_state(DepthStencilState::disabled());
    }

    if level >= Levels::FOUR {
        pipeline = pipeline.multisample_state(MultisampleState {
            rasterization_samples: SampleCount::Sample8,
            ..Default::default()
        })
    }
    let pipeline = pipeline
        .render_pass(Subpass::from(render_pass, 0).unwrap())
        .build(memory_allocator.device().clone())
        .unwrap();

    (pipeline, framebuffers)
}
