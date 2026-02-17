use std::time::{Duration, Instant};

use lite_room_application::{ApplicationService, SetEditCommand};
use lite_room_domain::{EditParams, ImageId};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};

const SLIDER_MIN: f32 = -5.0;
const SLIDER_MAX: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SliderField {
    Exposure,
    Contrast,
    Temperature,
    Tint,
    Highlights,
    Shadows,
}

#[derive(Debug, Clone, Copy)]
struct SliderSpec {
    field: SliderField,
    top: usize,
    color: u32,
}

struct DebouncedAutosave {
    debounce_ms: u64,
    dirty_since_ms: Option<u64>,
}

impl DebouncedAutosave {
    fn new(debounce_ms: u64) -> Self {
        Self {
            debounce_ms,
            dirty_since_ms: None,
        }
    }

    fn mark_dirty(&mut self, now_ms: u64) {
        self.dirty_since_ms = Some(now_ms);
    }

    fn should_flush(&self, now_ms: u64) -> bool {
        match self.dirty_since_ms {
            Some(since) => now_ms.saturating_sub(since) >= self.debounce_ms,
            None => false,
        }
    }

    fn clear(&mut self) {
        self.dirty_since_ms = None;
    }

    fn is_dirty(&self) -> bool {
        self.dirty_since_ms.is_some()
    }
}

pub fn launch_window(
    service: &ApplicationService,
    catalog_path: &str,
    cache_dir: &str,
    image_count: usize,
    image_id: Option<ImageId>,
    initial_params: EditParams,
) -> Result<(), String> {
    let width = 900;
    let height = 600;
    let sliders = slider_specs();

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
    window.limit_update_rate(Some(Duration::from_micros(16_000)));

    let mut buffer = vec![0x222222_u32; width * height];
    let start = Instant::now();
    let mut params = initial_params;
    let mut autosave = DebouncedAutosave::new(300);
    let mut active_drag: Option<SliderField> = None;
    let mut was_mouse_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mouse_down = window.get_mouse_down(MouseButton::Left);
        let mouse_pos = window.get_mouse_pos(MouseMode::Clamp);

        if mouse_down {
            if let Some((mouse_x, mouse_y)) = mouse_pos {
                if !was_mouse_down {
                    active_drag = slider_at_position(mouse_x, mouse_y, &sliders, width);
                }
                if let Some(field) = active_drag {
                    if update_param_from_mouse(&mut params, field, mouse_x, width) {
                        let now_ms = start.elapsed().as_millis() as u64;
                        autosave.mark_dirty(now_ms);
                    }
                }
            }
        } else {
            active_drag = None;
        }

        was_mouse_down = mouse_down;

        let now_ms = start.elapsed().as_millis() as u64;
        if autosave.should_flush(now_ms) {
            if let Some(active_image_id) = image_id {
                persist_edit(service, active_image_id, params)?;
            }
            autosave.clear();
        }

        draw_background(&mut buffer, width, height);
        draw_sliders(&mut buffer, width, &sliders, params);

        if let Some((mouse_x, mouse_y)) = mouse_pos {
            if let Some(hovered) = slider_at_position(mouse_x, mouse_y, &sliders, width) {
                draw_slider_hover(&mut buffer, width, hovered, &sliders);
            }
        }

        window.set_title(&build_window_title(
            catalog_path,
            cache_dir,
            image_count,
            image_id,
            params,
        ));

        window
            .update_with_buffer(&buffer, width, height)
            .map_err(|error| format!("failed to update UI window: {error}"))?;
    }

    if autosave.is_dirty() {
        if let Some(active_image_id) = image_id {
            persist_edit(service, active_image_id, params)?;
        }
    }

    Ok(())
}

fn persist_edit(
    service: &ApplicationService,
    image_id: ImageId,
    params: EditParams,
) -> Result<(), String> {
    service
        .set_edit(SetEditCommand { image_id, params })
        .map_err(|error| format!("autosave failed: {error}"))
}

fn slider_specs() -> [SliderSpec; 6] {
    [
        SliderSpec {
            field: SliderField::Exposure,
            top: 110,
            color: 0xE07A5F,
        },
        SliderSpec {
            field: SliderField::Contrast,
            top: 180,
            color: 0x81B29A,
        },
        SliderSpec {
            field: SliderField::Temperature,
            top: 250,
            color: 0xF2CC8F,
        },
        SliderSpec {
            field: SliderField::Tint,
            top: 320,
            color: 0x3D405B,
        },
        SliderSpec {
            field: SliderField::Highlights,
            top: 390,
            color: 0xBFC0C0,
        },
        SliderSpec {
            field: SliderField::Shadows,
            top: 460,
            color: 0x6D597A,
        },
    ]
}

fn draw_background(buffer: &mut [u32], width: usize, height: usize) {
    for y in 0..height {
        let row_color = if y % 40 < 20 { 0x202020 } else { 0x262626 };
        for x in 0..width {
            buffer[y * width + x] = row_color;
        }
    }
}

fn draw_sliders(buffer: &mut [u32], width: usize, sliders: &[SliderSpec], params: EditParams) {
    for slider in sliders {
        draw_slider_track(buffer, width, slider.top);
        let value = get_param_value(params, slider.field);
        let x = value_to_x(value, width);
        draw_slider_knob(buffer, width, x, slider.top, slider.color);
    }
}

fn draw_slider_track(buffer: &mut [u32], width: usize, top: usize) {
    let left = slider_left(width);
    let right = slider_right(width);
    let center_y = top + 16;
    for y in center_y - 3..=center_y + 3 {
        for x in left..=right {
            set_pixel(buffer, width, x, y, 0x4A4A4A);
        }
    }
}

fn draw_slider_knob(buffer: &mut [u32], width: usize, x: usize, top: usize, color: u32) {
    for y in top..top + 32 {
        for dx in 0..14 {
            let px = x.saturating_sub(7).saturating_add(dx);
            set_pixel(buffer, width, px, y, color);
        }
    }
}

fn draw_slider_hover(buffer: &mut [u32], width: usize, field: SliderField, sliders: &[SliderSpec]) {
    if let Some(spec) = sliders.iter().find(|spec| spec.field == field) {
        let left = slider_left(width);
        let right = slider_right(width);
        let top = spec.top.saturating_sub(6);
        let bottom = spec.top + 38;
        for x in left..=right {
            set_pixel(buffer, width, x, top, 0xFFFFFF);
            set_pixel(buffer, width, x, bottom, 0xFFFFFF);
        }
        for y in top..=bottom {
            set_pixel(buffer, width, left, y, 0xFFFFFF);
            set_pixel(buffer, width, right, y, 0xFFFFFF);
        }
    }
}

fn slider_left(width: usize) -> usize {
    width / 6
}

fn slider_right(width: usize) -> usize {
    width - (width / 6)
}

fn slider_at_position(
    mouse_x: f32,
    mouse_y: f32,
    sliders: &[SliderSpec],
    width: usize,
) -> Option<SliderField> {
    let x = mouse_x.max(0.0) as usize;
    let y = mouse_y.max(0.0) as usize;
    let left = slider_left(width);
    let right = slider_right(width);
    if x < left || x > right {
        return None;
    }
    sliders
        .iter()
        .find(|spec| y >= spec.top.saturating_sub(6) && y <= spec.top + 38)
        .map(|spec| spec.field)
}

fn update_param_from_mouse(
    params: &mut EditParams,
    field: SliderField,
    mouse_x: f32,
    width: usize,
) -> bool {
    let updated_value = x_to_value(mouse_x, width);
    let slot = match field {
        SliderField::Exposure => &mut params.exposure,
        SliderField::Contrast => &mut params.contrast,
        SliderField::Temperature => &mut params.temperature,
        SliderField::Tint => &mut params.tint,
        SliderField::Highlights => &mut params.highlights,
        SliderField::Shadows => &mut params.shadows,
    };
    if (*slot - updated_value).abs() < 0.0001 {
        return false;
    }
    *slot = updated_value;
    true
}

fn value_to_x(value: f32, width: usize) -> usize {
    let left = slider_left(width) as f32;
    let right = slider_right(width) as f32;
    let clamped = value.clamp(SLIDER_MIN, SLIDER_MAX);
    let t = (clamped - SLIDER_MIN) / (SLIDER_MAX - SLIDER_MIN);
    (left + t * (right - left)).round() as usize
}

fn x_to_value(x: f32, width: usize) -> f32 {
    let left = slider_left(width) as f32;
    let right = slider_right(width) as f32;
    let clamped = x.clamp(left, right);
    let t = (clamped - left) / (right - left);
    SLIDER_MIN + t * (SLIDER_MAX - SLIDER_MIN)
}

fn get_param_value(params: EditParams, field: SliderField) -> f32 {
    match field {
        SliderField::Exposure => params.exposure,
        SliderField::Contrast => params.contrast,
        SliderField::Temperature => params.temperature,
        SliderField::Tint => params.tint,
        SliderField::Highlights => params.highlights,
        SliderField::Shadows => params.shadows,
    }
}

fn set_pixel(buffer: &mut [u32], width: usize, x: usize, y: usize, color: u32) {
    let height = buffer.len() / width;
    if x < width && y < height {
        buffer[y * width + x] = color;
    }
}

fn field_name(field: SliderField) -> &'static str {
    match field {
        SliderField::Exposure => "exposure",
        SliderField::Contrast => "contrast",
        SliderField::Temperature => "temperature",
        SliderField::Tint => "tint",
        SliderField::Highlights => "highlights",
        SliderField::Shadows => "shadows",
    }
}

fn build_slider_status(params: EditParams) -> String {
    let fields = [
        SliderField::Exposure,
        SliderField::Contrast,
        SliderField::Temperature,
        SliderField::Tint,
        SliderField::Highlights,
        SliderField::Shadows,
    ];

    fields
        .iter()
        .map(|field| {
            format!(
                "{} {:.2}",
                field_name(*field),
                get_param_value(params, *field)
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn build_window_title(
    catalog_path: &str,
    cache_dir: &str,
    image_count: usize,
    image_id: Option<ImageId>,
    params: EditParams,
) -> String {
    match image_id {
        Some(image_id) => format!(
            "lite-room | catalog={} | cache={} | images={} | edit image={} | drag sliders | {} | esc quit",
            catalog_path,
            cache_dir,
            image_count,
            image_id.get(),
            build_slider_status(params)
        ),
        None => format!(
            "lite-room | catalog={} | cache={} | images={} | no image to edit | esc quit",
            catalog_path, cache_dir, image_count
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debounce_flushes_after_threshold() {
        let mut debounce = DebouncedAutosave::new(300);
        debounce.mark_dirty(100);
        assert!(!debounce.should_flush(399));
        assert!(debounce.should_flush(400));
    }

    #[test]
    fn x_and_value_mapping_roundtrip() {
        let width = 900;
        let original = 2.5;
        let x = value_to_x(original, width) as f32;
        let back = x_to_value(x, width);
        assert!((original - back).abs() < 0.05);
    }

    #[test]
    fn mouse_update_changes_expected_field() {
        let mut params = EditParams::default();
        let changed = update_param_from_mouse(&mut params, SliderField::Exposure, 700.0, 900);
        assert!(changed);
        assert!(params.exposure > 0.0);
        assert_eq!(params.contrast, 0.0);
    }
}
