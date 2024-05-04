use std::sync::Arc;

fn main() {
    let event_queue = {
        use winit::event_loop::{ControlFlow, EventLoop};

        let event_queue = EventLoop::new().unwrap();
        event_queue.set_control_flow(ControlFlow::Poll);
        event_queue
    };
    let instance = {
        use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
        use vulkano::VulkanLibrary;

        let library = VulkanLibrary::new().unwrap();
        let required_extensions = Surface::required_extensions(&event_queue);
        println!("Required extensions: {:?}", required_extensions);
        Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .unwrap()
    };

    let window = {
        use winit::{dpi::PhysicalSize, window::WindowBuilder};

        let window = WindowBuilder::new()
            .with_title("vulkano tutorial")
            .with_inner_size(PhysicalSize {
                width: 480,
                height: 320,
            })
            .build(&event_queue)
            .unwrap();
        Arc::new(window)
    };

    use vulkano::device::{physical::PhysicalDeviceType, DeviceExtensions, QueueFlags};
    use vulkano::swapchain::Surface;
    use vulkano::Version;
    
    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();
    let mut device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    println!("Device extensions: {:?}", device_extensions);
    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| {
            p.api_version() >= Version::V1_3 || p.supported_extensions().khr_dynamic_rendering
        })
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.intersects(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &surface).unwrap_or(false)
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
        .expect("no suitable physical device found");
    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );
    println!("Using vulkan api: {}", physical_device.api_version());
    if physical_device.api_version() < Version::V1_3 {
        device_extensions.khr_dynamic_rendering = true;
    }

    let (device, mut queues) = {
        use vulkano::device::{Device, DeviceCreateInfo, Features, QueueCreateInfo};

        Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                enabled_features: Features {
                    dynamic_rendering: true,
                    ..Features::empty()
                },
                ..Default::default()
            },
        )
        .unwrap()
    };

    // Since we can request multiple queues, the `queues` variable is in fact an iterator. We only
    // use one queue in this example, so we just retrieve the first and only element of the
    // iterator.
    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        use vulkano::image::ImageUsage;
        use vulkano::swapchain::{Swapchain, SwapchainCreateInfo};

        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .unwrap();
        let image_format = device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0;
        Swapchain::new(
            device.clone(),
            surface,
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count.max(2),
                image_format,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .into_iter()
                    .next()
                    .unwrap(),
                ..Default::default()
            },
        )
        .unwrap()
    };

    use vulkano::memory::allocator::StandardMemoryAllocator;

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
    use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};

    let vertex_buffer = Buffer::from_iter(
        memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        MY_VERTEX_DATA,
    )
    .unwrap();

    let pipeline = {
        use vulkano::pipeline::{
            graphics::vertex_input::VertexDefinition,
            layout::PipelineDescriptorSetLayoutCreateInfo, PipelineLayout,
            PipelineShaderStageCreateInfo,
        };

        let vs = my_vs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = my_fs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let vertex_input_state = MyVertex::per_vertex()
            .definition(&vs.info().input_interface)
            .unwrap();
        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();

        use vulkano::pipeline::graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            subpass::PipelineRenderingCreateInfo,
            viewport::ViewportState,
            GraphicsPipeline, GraphicsPipelineCreateInfo,
        };
        use vulkano::pipeline::DynamicState;

        let subpass = PipelineRenderingCreateInfo {
            color_attachment_formats: vec![Some(swapchain.image_format())],
            ..Default::default()
        };

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.color_attachment_formats.len() as u32,
                    ColorBlendAttachmentState::default(),
                )),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap()
    };

    use vulkano::pipeline::graphics::viewport::Viewport;

    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [0.0, 0.0],
        depth_range: 0.0..=1.0,
    };

    let mut attachment_image_views = {
        use vulkano::image::view::ImageView;

        let extent = images[0].extent();
        viewport.extent = [extent[0] as f32, extent[1] as f32];
        images
            .iter()
            .map(|image| ImageView::new_default(image.clone()).unwrap())
            .collect::<Vec<_>>()
    };

    use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;

    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        Default::default(),
    ));

    let mut recreate_swapchain = false;

    use vulkano::sync::{self, GpuFuture};

    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    _ = event_queue.run(move |event, window_target| {
        use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
        use winit::keyboard::{KeyCode, PhysicalKey};

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                window_target.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                recreate_swapchain = true;
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let image_extent: [u32; 2] = window.inner_size().into();
                if image_extent.contains(&0) {
                    return;
                }

                previous_frame_end.as_mut().unwrap().cleanup_finished();

                if recreate_swapchain {
                    use vulkano::image::view::ImageView;
                    use vulkano::swapchain::SwapchainCreateInfo;

                    let (new_swapchain, new_images) = swapchain
                        .recreate(SwapchainCreateInfo {
                            image_extent,
                            ..swapchain.create_info()
                        })
                        .expect("failed to recreate swapchain");
                    swapchain = new_swapchain;
                    attachment_image_views = {
                        let extent = new_images[0].extent();
                        viewport.extent = [extent[0] as f32, extent[1] as f32];
                        new_images
                            .iter()
                            .map(|image| ImageView::new_default(image.clone()).unwrap())
                            .collect::<Vec<_>>()
                    };
                    recreate_swapchain = false;
                }

                use vulkano::{Validated, VulkanError};

                let (image_index, suboptimal, acquire_future) =
                    match vulkano::swapchain::acquire_next_image(swapchain.clone(), None)
                        .map_err(Validated::unwrap)
                    {
                        Ok(r) => r,
                        Err(VulkanError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("failed to acquire next image: {e}"),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                use vulkano::command_buffer::{auto::AutoCommandBufferBuilder, CommandBufferUsage};

                let mut builder = AutoCommandBufferBuilder::primary(
                    command_buffer_allocator.as_ref(),
                    queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                use vulkano::command_buffer::{RenderingAttachmentInfo, RenderingInfo};
                use vulkano::render_pass::{AttachmentLoadOp, AttachmentStoreOp};

                builder
                    .begin_rendering(RenderingInfo {
                        color_attachments: vec![Some(RenderingAttachmentInfo {
                            load_op: AttachmentLoadOp::Clear,
                            store_op: AttachmentStoreOp::Store,
                            clear_value: Some([0.2, 0.2, 0.2, 1.0].into()),
                            ..RenderingAttachmentInfo::image_view(
                                attachment_image_views[image_index as usize].clone(),
                            )
                        })],
                        ..Default::default()
                    })
                    .unwrap()
                    .set_viewport(0, [viewport.clone()].into_iter().collect())
                    .unwrap()
                    .bind_pipeline_graphics(pipeline.clone())
                    .unwrap()
                    .bind_vertex_buffers(0, vertex_buffer.clone())
                    .unwrap()
                    .draw(vertex_buffer.len() as u32, 1, 0, 0)
                    .unwrap();

                builder.end_rendering().unwrap();

                let command_buffer = builder.build().unwrap();

                use vulkano::swapchain::SwapchainPresentInfo;

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

                match future.map_err(Validated::unwrap) {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(VulkanError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                    Err(e) => {
                        println!("failed to flush future: {e}");
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                }
            }
            Event::AboutToWait => window.request_redraw(),
            _ => (),
        }
    });
}

use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[repr(C)]
#[derive(Debug, Clone, Copy, BufferContents, Vertex)]
struct MyVertex {
    #[format(R32G32_SFLOAT)]
    pos: [f32; 2],
    #[format(R32G32B32_SFLOAT)]
    col: [f32; 3],
}

static MY_VERTEX_DATA: [MyVertex; 3] = [
    MyVertex {
        pos: [0.8, 0.0],
        col: [1.0, 0.0, 0.0],
    },
    MyVertex {
        pos: [0.0, 0.8],
        col: [0.0, 1.0, 0.0],
    },
    MyVertex {
        pos: [-0.8, -0.8],
        col: [0.0, 0.0, 1.0],
    },
];

mod my_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450
        
            layout(location = 0) in vec2 pos;
            layout(location = 1) in vec3 col;
            layout(location = 0) out vec3 v_color;
        
            void main() {
                gl_Position = vec4(pos, 0.0, 1.0);
                v_color = col;
            }
        ",
    }
}

mod my_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450

            layout(location = 0) in vec3 v_color;
            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(v_color, 1.0);
            }
        ",
    }
}
