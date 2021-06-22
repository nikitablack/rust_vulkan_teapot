mod teapot_data;
mod vulkan;

use ash::version::DeviceV1_0;
use vulkan::VulkanData;
use vulkan_base::VulkanBase;

const CONCURRENT_RESOURCE_COUNT: u32 = 2;

fn main() {
    // logger
    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = vec![simplelog::TermLogger::new(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )];
    if let Ok(file) = std::fs::File::create("log.txt") {
        loggers.push(simplelog::WriteLogger::new(
            simplelog::LevelFilter::Info,
            simplelog::Config::default(),
            file,
        ));
    }

    let _ = simplelog::CombinedLogger::init(loggers);

    // window
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Teapot")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .with_min_inner_size(winit::dpi::PhysicalSize::new(100.0, 100.0))
        .build(&event_loop)
        .unwrap();

    // vulkan base
    let enable_debug_utils = true;
    let device_extensions = vec![ash::extensions::khr::Swapchain::name()];
    let instance_extensions =
        vulkan::get_required_instance_extensions(&window, enable_debug_utils).unwrap();

    let mut vk_base = match VulkanBase::new(
        &window,
        &instance_extensions,
        &device_extensions,
        enable_debug_utils,
    ) {
        Ok(vk_base) => vk_base,
        Err(msg) => {
            log::error!("{}", msg);
            panic!("{}", msg);
        }
    };

    // vulkan data
    let mut vk_data = match VulkanData::new(&vk_base) {
        Ok(vk_data) => vk_data,
        Err(msg) => {
            log::error!("{}", msg);
            panic!("{}", msg);
        }
    };

    // loop
    let mut app_exit = false;
    let start_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        use winit::event::ElementState;
        use winit::event::Event;
        use winit::event::KeyboardInput;
        use winit::event::VirtualKeyCode;
        use winit::event::WindowEvent;
        use winit::event_loop::ControlFlow;

        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;

                log::info!("exit requested");

                unsafe {
                    let _ = vk_base.device.device_wait_idle();
                }

                vk_data.clean(&vk_base);
                vk_base.clean();

                app_exit = true;
            }

            Event::MainEventsCleared => {
                if app_exit {
                    return;
                }

                // do nothing if window is minimized
                if window.inner_size().width == 0 && window.inner_size().height == 0 {
                    return;
                }

                if vk_data.should_resize {
                    vk_data.should_resize = false;

                    log::info!("handling resize");

                    if let Err(msg) = vk_base.resize(&window) {
                        panic!("{}", msg);
                    }

                    if let Err(msg) = vk_data.resize(&vk_base) {
                        panic!("{}", msg);
                    }
                }

                if let Err(msg) = vulkan::draw(
                    &mut vk_data,
                    &vk_base,
                    (std::time::Instant::now() - start_time).as_secs_f32(),
                ) {
                    panic!(msg);
                }

                vk_data.curr_resource_index =
                    (vk_data.curr_resource_index + 1) % CONCURRENT_RESOURCE_COUNT;
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                log::info!("resize requested {:?}", physical_size);
                vk_data.should_resize = true;
            }

            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(virtual_code),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => match virtual_code {
                VirtualKeyCode::Space => {
                    vk_data.is_wireframe_mode = !vk_data.is_wireframe_mode;
                }
                VirtualKeyCode::Plus | VirtualKeyCode::NumpadAdd => {
                    vk_data.tesselation_level += 0.1f32;
                    vk_data.tesselation_level = vk_data.tesselation_level.min(64.0);
                }
                VirtualKeyCode::Minus | VirtualKeyCode::NumpadSubtract => {
                    vk_data.tesselation_level -= 0.1f32;
                    vk_data.tesselation_level = vk_data.tesselation_level.max(1.0);
                }
                _ => (),
            },

            _ => {}
        }
    });
}
