pub fn render_ui<'img_ctx>(
    vulkan_data: &vulkan_base::VulkanBaseData,
    imgui_ui: imgui::Ui<'img_ctx>,
) -> (&'img_ctx imgui::DrawData, common::UiResult) {
    let mut ui_result = common::UiResult::None;

    let window_width = vulkan_data.surface_extent.width as f32;
    let window_height = vulkan_data.surface_extent.height as f32;
    let button_width = window_width * 0.2;
    let button_height = button_width * 0.5;
    let window_padding_h = (window_width - button_width) * 0.5;
    let window_padding_v = (window_height - button_height) * 0.5;

    let styles = imgui_ui.push_style_vars(&[
        imgui::StyleVar::WindowBorderSize(0.0),
        imgui::StyleVar::WindowRounding(0.0),
        imgui::StyleVar::WindowPadding([0.0, 0.0]),
    ]);

    imgui::Window::new(imgui::im_str!("Title Screen"))
        .size([window_width, window_height], imgui::Condition::Always)
        .position([0.0, 0.0], imgui::Condition::Always)
        .flags(imgui::WindowFlags::NO_DECORATION | imgui::WindowFlags::NO_BACKGROUND)
        .build(&imgui_ui, || {
            imgui_ui.set_cursor_pos([window_padding_h, window_padding_v]);
            if imgui_ui.button(imgui::im_str!("Quit"), [button_width, button_height]) == true {
                ui_result = common::UiResult::Quit;
            }
        });

    styles.pop(&imgui_ui);

    //imgui_ui.show_demo_window(&mut true);

    (imgui_ui.render(), ui_result)
}
