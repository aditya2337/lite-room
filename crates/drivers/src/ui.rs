use minifb::{Key, Window, WindowOptions};

pub fn launch_window(
    catalog_path: &str,
    cache_dir: &str,
    image_count: usize,
) -> Result<(), String> {
    let width = 900;
    let height = 600;

    let mut window = Window::new(
        &format!(
            "lite-room | catalog={} | cache={} | images={}",
            catalog_path, cache_dir, image_count
        ),
        width,
        height,
        WindowOptions::default(),
    )
    .map_err(|error| format!("failed to start UI window: {error}"))?;

    let mut buffer = vec![0x222222_u32; width * height];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, width, height)
            .map_err(|error| format!("failed to update UI window: {error}"))?;

        for y in 0..height {
            let row_color = if y % 40 < 20 { 0x202020 } else { 0x262626 };
            for x in 0..width {
                buffer[y * width + x] = row_color;
            }
        }
    }

    Ok(())
}
