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
    let device_extensions = vec![ash::extensions::khr::Swapchain::name()];
    let instance_extensions = vulkan::get_required_instance_extensions(&window).unwrap();

    let mut vk_base = match VulkanBase::new(&window, &instance_extensions, &device_extensions) {
        Ok(vk_base) => Some(vk_base),
        Err(msg) => {
            log::error!("{}", msg);
            panic!("{}", msg);
        }
    };

    // vulkan data
    let mut vk_data = match VulkanData::new(vk_base.as_mut().unwrap()) {
        Ok(vk_data) => Some(vk_data),
        Err(msg) => {
            log::error!("{}", msg);
            let vk_base = vk_base.unwrap();
            vk_base.clean();
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

                vulkan::vulkan_clean(&mut vk_base, &mut vk_data);

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

                let vk_base_ref = vk_base.as_mut().unwrap();
                let vk_data_ref = vk_data.as_mut().unwrap();

                if vk_data_ref.should_resize {
                    vk_data_ref.should_resize = false;

                    log::info!("handling resize");

                    if let Err(msg) = vk_base_ref.resize(&window) {
                        log::error!("{}", msg);
                        vulkan::vulkan_clean(&mut vk_base, &mut vk_data);
                        app_exit = true;
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    if let Err(msg) = vk_data_ref.resize(&vk_base_ref) {
                        log::error!("{}", msg);
                        vulkan::vulkan_clean(&mut vk_base, &mut vk_data);
                        app_exit = true;
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }

                // TODO draw
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                log::info!("resize requested {:?}", physical_size);

                let vk_data = vk_data.as_mut().unwrap();
                vk_data.should_resize = true;
            }

            _ => {}
        }
    });
}
