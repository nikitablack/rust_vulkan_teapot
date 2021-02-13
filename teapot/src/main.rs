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
        .with_title("Teapot")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    // loop
    let mut app_exit = false;
    let mut window_resized = false;

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

                // TODO cleanup

                app_exit = true;

                log::info!("exit requested");
            }

            Event::MainEventsCleared => {
                if app_exit {
                    return;
                }

                // do nothing if window is minimized
                if window.inner_size().width == 0 && window.inner_size().height == 0 {
                    return;
                }

                if window_resized {
                    window_resized = false;

                    // TODO handle resize
                }

                // TODO draw
            }

            Event::WindowEvent {
                event: WindowEvent::Resized { .. },
                ..
            } => {
                window_resized = true;
                log::info!("resize requested");
            }

            _ => {}
        }
    });
}
