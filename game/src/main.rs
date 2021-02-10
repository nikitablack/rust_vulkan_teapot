use core::panic;

mod vulkan;

fn main() {
    // logger
    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = vec![simplelog::TermLogger::new(
        simplelog::LevelFilter::Trace,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
    )];
    if let Ok(file) = std::fs::File::create("log.txt") {
        loggers.push(simplelog::WriteLogger::new(
            simplelog::LevelFilter::Trace,
            simplelog::Config::default(),
            file,
        ));
    }

    let _ = simplelog::CombinedLogger::init(loggers);

    // window
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Game")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    // imgui
    let mut imgui_context = imgui::Context::create();
    imgui_context.set_ini_filename(None);
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
    platform.attach_window(
        imgui_context.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Locked(1.0),
    );

    // vulkan base
    let device_extensions = vec![ash::extensions::khr::Swapchain::name()];
    let instance_extensions = vulkan::get_required_instance_extensions(&window).unwrap();

    let mut vulkan_base_data = match vulkan_base::init_vulkan(
        &window,
        &instance_extensions,
        &device_extensions,
        true,
        0,
    ) {
        Ok(vbd) => vbd,
        Err(msg) => {
            log::error!("{}", msg);
            panic!(&msg);
        }
    };

    // title screen
    let mut title_screen_data = {
        let mut imgui_fonts = imgui_context.fonts();
        let font_texture = imgui_fonts.build_rgba32_texture();
        match title_screen::init_vulkan(&mut vulkan_base_data, &font_texture) {
            Ok(tsd) => tsd,
            Err(msg) => {
                log::error!("{}", msg);
                panic!(&msg);
            }
        }
    };

    // loop
    let mut app_exit = false;
    let mut window_resized = false;
    let mut last_frame = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        use winit::event::Event;
        use winit::event::WindowEvent;
        use winit::event_loop::ControlFlow;

        *control_flow = ControlFlow::Poll;

        match event {
            Event::NewEvents(_) => {
                imgui_context
                    .io_mut()
                    .update_delta_time(std::time::Instant::now() - last_frame);
                last_frame = std::time::Instant::now();
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;

                title_screen::vulkan::clear_title_screen_data(
                    &mut title_screen_data,
                    &mut vulkan_base_data,
                );
                vulkan_base::clear_vulkan_base(&mut vulkan_base_data);

                app_exit = true;
            }

            Event::MainEventsCleared => {
                if app_exit {
                    return;
                }

                if window.inner_size().width == 0 && window.inner_size().height == 0 {
                    return;
                }

                if window_resized {
                    window_resized = false;

                    title_screen::vulkan::handle_window_resize(
                        &mut title_screen_data,
                        &mut vulkan_base_data,
                        &window,
                        &mut imgui_context,
                    )
                    .unwrap();
                }

                platform
                    .prepare_frame(imgui_context.io_mut(), &window)
                    .expect("Failed to prepare frame");

                let imgui_ui = imgui_context.frame();
                platform.prepare_render(&imgui_ui, &window);

                let (ui_draw_data, ui_result) =
                    title_screen::render_ui(&vulkan_base_data, imgui_ui);

                match ui_result {
                    common::UiResult::Quit => {
                        *control_flow = ControlFlow::Exit;

                        title_screen::vulkan::clear_title_screen_data(
                            &mut title_screen_data,
                            &mut vulkan_base_data,
                        );
                        vulkan_base::clear_vulkan_base(&mut vulkan_base_data);

                        app_exit = true;

                        return;
                    }
                    common::UiResult::None => (),
                }

                if let Err(msg) =
                    title_screen::draw(&mut title_screen_data, &vulkan_base_data, &ui_draw_data)
                {
                    panic!(&msg);
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Resized { .. },
                ..
            } => {
                window_resized = true;
                platform.handle_event(imgui_context.io_mut(), &window, &event);
            }

            event => {
                platform.handle_event(imgui_context.io_mut(), &window, &event);
            }
        }
    });
}
