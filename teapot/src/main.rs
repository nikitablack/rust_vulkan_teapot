mod teapot_data;
mod vulkan;

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
    let device_extensions = vec![];
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
    let vk_data = match VulkanData::new(&vk_base) {
        Ok(vk_data) => vk_data,
        Err(msg) => {
            log::error!("{}", msg);
            panic!("{}", msg);
        }
    };

    // loop
    let mut app_exit = false;

    event_loop.run(move |event, _, control_flow| {
        use winit::event::Event;
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

                // TODO draw
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                log::info!("resize requested {:?}", physical_size);
            }

            _ => {}
        }
    });
}
