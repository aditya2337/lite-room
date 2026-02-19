use std::time::{Duration, Instant};

use font8x8::UnicodeFonts;
use image::io::Reader as ImageReader;
use lite_room_application::{
    ApplicationService, ListImagesCommand, PollPreviewCommand, PreviewMetricsQuery,
    SetEditCommand, ShowEditCommand, SubmitPreviewCommand,
};
use lite_room_domain::{EditParams, ImageId, PreviewFrame, PreviewMetrics};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};

const SLIDER_MIN: f32 = -5.0;
const SLIDER_MAX: f32 = 5.0;
const WINDOW_WIDTH: usize = 1120;
const WINDOW_HEIGHT: usize = 700;
const CANVAS_MARGIN: usize = 24;
const HEADER_TOP: usize = 20;
const HEADER_HEIGHT: usize = 56;
const WORKAREA_TOP: usize = 94;
const WORKAREA_BOTTOM_MARGIN: usize = 28;
const SPLIT_GUTTER: usize = 24;
const CONTROL_PANEL_WIDTH: usize = 300;
const CONTROL_INSET: usize = 18;
const SLIDER_HEIGHT: usize = 54;
const SLIDER_GAP: usize = 14;

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

#[derive(Debug, Clone)]
struct PreviewCanvas {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
struct TitleTelemetry<'a> {
    latest_frame: Option<&'a PreviewFrame>,
    metrics: &'a PreviewMetrics,
    preview_canvas: Option<&'a PreviewCanvas>,
    image_index: Option<(usize, usize)>,
    focused_slider: Option<SliderField>,
}

pub fn launch_window(
    service: &ApplicationService,
    catalog_path: &str,
    cache_dir: &str,
    image_count: usize,
    image_id: Option<ImageId>,
    image_path: Option<String>,
    initial_params: EditParams,
) -> Result<(), String> {
    let width = WINDOW_WIDTH;
    let height = WINDOW_HEIGHT;
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
    let mut latest_frame: Option<PreviewFrame> = None;
    let mut active_image_id = image_id;
    let mut active_image_path = image_path;
    let mut preview = load_preview_canvas(active_image_path.as_deref(), width, height);
    let catalog_images = service
        .list_images(ListImagesCommand)
        .map_err(|error| format!("list images failed: {error}"))?;
    let mut active_index = active_image_id.and_then(|id| {
        catalog_images
            .iter()
            .enumerate()
            .find(|(_, image)| image.id == id)
            .map(|(index, _)| index)
    });

    if let Some(id) = active_image_id {
        submit_preview(service, id, params, width as u32, height as u32)?;
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let go_prev = window.is_key_pressed(Key::Left, KeyRepeat::No);
        let go_next = window.is_key_pressed(Key::Right, KeyRepeat::No);
        if !catalog_images.is_empty() && (go_prev || go_next) {
            if autosave.is_dirty() {
                if let Some(id) = active_image_id {
                    persist_edit(service, id, params)?;
                }
                autosave.clear();
            }

            let len = catalog_images.len();
            let current = active_index.unwrap_or(0);
            let next = if go_next {
                (current + 1) % len
            } else {
                (current + len - 1) % len
            };

            let next_image = &catalog_images[next];
            active_index = Some(next);
            active_image_id = Some(next_image.id);
            active_image_path = Some(next_image.file_path.clone());
            params = service
                .show_edit(ShowEditCommand {
                    image_id: next_image.id,
                })
                .map_err(|error| format!("show-edit failed during image switch: {error}"))?;
            preview = load_preview_canvas(active_image_path.as_deref(), width, height);
            latest_frame = None;
            submit_preview(service, next_image.id, params, width as u32, height as u32)?;
        }

        let mouse_down = window.get_mouse_down(MouseButton::Left);
        let mouse_pos = window.get_mouse_pos(MouseMode::Clamp);
        let hovered_slider = mouse_pos
            .and_then(|(mouse_x, mouse_y)| slider_at_position(mouse_x, mouse_y, &sliders, width));

        if mouse_down {
            if let Some((mouse_x, _)) = mouse_pos {
                if !was_mouse_down {
                    active_drag = hovered_slider;
                }
                if let Some(field) = active_drag {
                    if update_param_from_mouse(&mut params, field, mouse_x, width) {
                        let now_ms = start.elapsed().as_millis() as u64;
                        autosave.mark_dirty(now_ms);
                        if let Some(id) = active_image_id {
                            submit_preview(service, id, params, width as u32, height as u32)?;
                        }
                    }
                }
            }
        } else {
            active_drag = None;
        }

        was_mouse_down = mouse_down;

        let now_ms = start.elapsed().as_millis() as u64;
        if autosave.should_flush(now_ms) {
            if let Some(id) = active_image_id {
                persist_edit(service, id, params)?;
            }
            autosave.clear();
        }

        draw_background(&mut buffer, width, height);
        draw_header(&mut buffer, width);
        draw_preview_shadow(&mut buffer, width, height);
        draw_preview_panel(&mut buffer, width, height, &preview);
        draw_sliders(
            &mut buffer,
            width,
            height,
            &sliders,
            params,
            active_drag.or(hovered_slider),
            active_index.map(|index| (index + 1, catalog_images.len())),
        );

        if let Some(frame) = service
            .poll_preview(PollPreviewCommand)
            .map_err(|error| format!("preview poll failed: {error}"))?
        {
            preview = Some(preview_canvas_from_frame(&frame, width, height));
            latest_frame = Some(frame);
        }
        let metrics = service
            .preview_metrics(PreviewMetricsQuery)
            .map_err(|error| format!("preview metrics failed: {error}"))?;

        if let Some(hovered) = hovered_slider {
            draw_slider_hover(&mut buffer, width, hovered, &sliders);
        }

        window.set_title(&build_window_title(
            catalog_path,
            cache_dir,
            image_count,
            active_image_id,
            params,
            TitleTelemetry {
                latest_frame: latest_frame.as_ref(),
                metrics: &metrics,
                preview_canvas: preview.as_ref(),
                image_index: active_index.map(|index| (index + 1, catalog_images.len())),
                focused_slider: active_drag.or(hovered_slider),
            },
        ));

        window
            .update_with_buffer(&buffer, width, height)
            .map_err(|error| format!("failed to update UI window: {error}"))?;
    }

    if autosave.is_dirty() {
        if let Some(id) = active_image_id {
            persist_edit(service, id, params)?;
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

fn submit_preview(
    service: &ApplicationService,
    image_id: ImageId,
    params: EditParams,
    target_width: u32,
    target_height: u32,
) -> Result<(), String> {
    service
        .submit_preview(SubmitPreviewCommand {
            image_id,
            params,
            target_width,
            target_height,
        })
        .map_err(|error| format!("preview submit failed: {error}"))
}

fn load_preview_canvas(
    image_path: Option<&str>,
    window_width: usize,
    window_height: usize,
) -> Option<PreviewCanvas> {
    let path = image_path?;
    let image = ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;

    let source = image.to_rgb8();
    let src_width = source.width() as usize;
    let src_height = source.height() as usize;
    if src_width == 0 || src_height == 0 {
        return None;
    }

    let panel_left = preview_panel_left();
    let panel_right = preview_panel_right(window_width);
    let panel_top = preview_panel_top();
    let panel_bottom = preview_panel_bottom(window_height);
    let max_width = panel_right.saturating_sub(panel_left + 26);
    let max_height = panel_bottom.saturating_sub(panel_top + 26);
    if max_width == 0 || max_height == 0 {
        return None;
    }

    let scale = (max_width as f32 / src_width as f32).min(max_height as f32 / src_height as f32);
    let dst_width = ((src_width as f32 * scale).max(1.0)).round() as usize;
    let dst_height = ((src_height as f32 * scale).max(1.0)).round() as usize;

    let mut pixels = vec![0_u32; dst_width * dst_height];
    for y in 0..dst_height {
        let src_y = y * src_height / dst_height;
        for x in 0..dst_width {
            let src_x = x * src_width / dst_width;
            let pixel = source.get_pixel(src_x as u32, src_y as u32);
            let [r, g, b] = pixel.0;
            pixels[y * dst_width + x] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        }
    }

    Some(PreviewCanvas {
        width: dst_width,
        height: dst_height,
        pixels,
    })
}

fn draw_preview_panel(buffer: &mut [u32], width: usize, height: usize, preview: &Option<PreviewCanvas>) {
    let panel_left = preview_panel_left();
    let panel_top = preview_panel_top();
    let panel_right = preview_panel_right(width);
    let panel_bottom = preview_panel_bottom(height);

    fill_rect(
        buffer,
        width,
        panel_left,
        panel_top,
        panel_right.saturating_sub(panel_left),
        panel_bottom.saturating_sub(panel_top),
        0xFBFAF7,
    );
    draw_rect(
        buffer,
        width,
        panel_left,
        panel_top,
        panel_right.saturating_sub(panel_left),
        panel_bottom.saturating_sub(panel_top),
        0xC8B89F,
    );

    let stage_left = panel_left + 12;
    let stage_top = panel_top + 12;
    let stage_width = panel_right.saturating_sub(stage_left + 12);
    let stage_height = panel_bottom.saturating_sub(stage_top + 12);
    fill_rect(
        buffer,
        width,
        stage_left,
        stage_top,
        stage_width,
        stage_height,
        0x101010,
    );
    draw_rect(
        buffer,
        width,
        stage_left,
        stage_top,
        stage_width,
        stage_height,
        0x2D2D2D,
    );

    let Some(preview) = preview else {
        return;
    };

    let content_width = stage_width.saturating_sub(2);
    let content_height = stage_height.saturating_sub(2);
    let draw_width = preview.width.min(content_width);
    let draw_height = preview.height.min(content_height);
    let start_x = stage_left + 1 + (content_width.saturating_sub(draw_width)) / 2;
    let start_y = stage_top + 1 + (content_height.saturating_sub(draw_height)) / 2;

    for y in 0..draw_height {
        for x in 0..draw_width {
            let color = preview.pixels[y * preview.width + x];
            set_pixel(buffer, width, start_x + x, start_y + y, color);
        }
    }
}

fn preview_canvas_from_frame(
    frame: &PreviewFrame,
    window_width: usize,
    window_height: usize,
) -> PreviewCanvas {
    let src_width = frame.width as usize;
    let src_height = frame.height as usize;
    if src_width == 0 || src_height == 0 || frame.pixels.is_empty() {
        return PreviewCanvas {
            width: 1,
            height: 1,
            pixels: vec![0_u32],
        };
    }

    let panel_left = preview_panel_left();
    let panel_right = preview_panel_right(window_width);
    let panel_top = preview_panel_top();
    let panel_bottom = preview_panel_bottom(window_height);
    let max_width = panel_right.saturating_sub(panel_left + 26).max(1);
    let max_height = panel_bottom.saturating_sub(panel_top + 26).max(1);

    let scale = (max_width as f32 / src_width as f32).min(max_height as f32 / src_height as f32);
    let dst_width = ((src_width as f32 * scale).max(1.0)).round() as usize;
    let dst_height = ((src_height as f32 * scale).max(1.0)).round() as usize;

    let mut pixels = vec![0_u32; dst_width * dst_height];
    for y in 0..dst_height {
        let src_y = y * src_height / dst_height;
        for x in 0..dst_width {
            let src_x = x * src_width / dst_width;
            pixels[y * dst_width + x] = frame.pixels[src_y * src_width + src_x];
        }
    }

    PreviewCanvas {
        width: dst_width,
        height: dst_height,
        pixels,
    }
}

fn slider_specs() -> [SliderSpec; 6] {
    let start = control_panel_top() + 126;
    let stride = SLIDER_HEIGHT + SLIDER_GAP;
    [
        SliderSpec {
            field: SliderField::Exposure,
            top: start,
            color: 0xFF996C,
        },
        SliderSpec {
            field: SliderField::Contrast,
            top: start + stride,
            color: 0x9CD8BE,
        },
        SliderSpec {
            field: SliderField::Temperature,
            top: start + stride * 2,
            color: 0xFFD58F,
        },
        SliderSpec {
            field: SliderField::Tint,
            top: start + stride * 3,
            color: 0x8A95D8,
        },
        SliderSpec {
            field: SliderField::Highlights,
            top: start + stride * 4,
            color: 0xD8E2F0,
        },
        SliderSpec {
            field: SliderField::Shadows,
            top: start + stride * 5,
            color: 0xBEA6E8,
        },
    ]
}

fn draw_background(buffer: &mut [u32], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let t = y as f32 / height.max(1) as f32;
            let mut color = lerp_color(0xF7EFE0, 0xF2E1CC, t);
            if ((x + (y * 2)) / 36) % 2 == 0 {
                color = darken_color(color, 6);
            }
            if ((x * 3 + y) / 59) % 3 == 0 {
                color = lighten_color(color, 8);
            }
            buffer[y * width + x] = color;
        }
    }

    let vignette = 160usize;
    for y in 0..height {
        for x in 0..width {
            let dx = x.min(width.saturating_sub(1).saturating_sub(x));
            let dy = y.min(height.saturating_sub(1).saturating_sub(y));
            let edge = dx.min(dy);
            if edge < vignette {
                let strength = ((vignette - edge) as f32 / vignette as f32 * 20.0) as u8;
                let idx = y * width + x;
                buffer[idx] = darken_color(buffer[idx], strength);
            }
        }
    }
}

fn draw_sliders(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    sliders: &[SliderSpec],
    params: EditParams,
    focused_slider: Option<SliderField>,
    image_index: Option<(usize, usize)>,
) {
    draw_control_panel(buffer, width, height);
    draw_control_text(buffer, width, sliders, params, focused_slider, image_index);
    for slider in sliders {
        draw_slider_shell(buffer, width, slider.top);
        let value = get_param_value(params, slider.field);
        let x = value_to_x(value, width);
        draw_slider_track(buffer, width, slider.top, x, slider.color);
        draw_slider_knob(buffer, width, x, slider.top, slider.color);
        let label = format!("{} {:+.2}", slider_label(slider.field), value);
        draw_text(
            buffer,
            width,
            slider_left(width) + 8,
            slider.top + 6,
            &label,
            0x4A3E2E,
        );
    }
}

fn draw_control_text(
    buffer: &mut [u32],
    width: usize,
    sliders: &[SliderSpec],
    _params: EditParams,
    focused_slider: Option<SliderField>,
    image_index: Option<(usize, usize)>,
) {
    let left = control_panel_left(width);
    let top = control_panel_top();
    let image_text = image_index
        .map(|(current, total)| format!("IMAGE {}/{}", current, total))
        .unwrap_or_else(|| "IMAGE 0/0".to_string());
    draw_text(buffer, width, left + 22, top + 48, &image_text, 0x1B1F26);
    draw_text(
        buffer,
        width,
        left + 22,
        top + 64,
        "LEFT/RIGHT: SWITCH IMAGE",
        0x4A3E2E,
    );

    let focus_text = focused_slider
        .map(|field| format!("{}: {}", slider_label(field), slider_effect(field)))
        .unwrap_or_else(|| "HOVER A SLIDER TO SEE EFFECT".to_string());
    draw_text(buffer, width, left + 22, top + 80, &focus_text, 0x4A3E2E);

    if let Some(first) = sliders.first() {
        let y = first.top.saturating_sub(16);
        draw_text(buffer, width, slider_left(width) + 8, y, "SLIDER + VALUE", 0x6A5B47);
    }
}

fn draw_slider_shell(buffer: &mut [u32], width: usize, top: usize) {
    let left = slider_left(width);
    let right = slider_right(width);
    fill_rect(
        buffer,
        width,
        left,
        top,
        right.saturating_sub(left).saturating_add(1),
        SLIDER_HEIGHT,
        0xFAF6EE,
    );
    draw_rect(
        buffer,
        width,
        left,
        top,
        right.saturating_sub(left).saturating_add(1),
        SLIDER_HEIGHT,
        0xD8C7AD,
    );
}

fn draw_slider_track(buffer: &mut [u32], width: usize, top: usize, knob_x: usize, color: u32) {
    let left = slider_left(width);
    let right = slider_right(width);
    let center_y = top + (SLIDER_HEIGHT / 2);

    for y in center_y.saturating_sub(2)..=center_y + 2 {
        for x in left + 8..right.saturating_sub(8) {
            set_pixel(buffer, width, x, y, 0xB8A58D);
        }
    }

    let center_x = value_to_x(0.0, width);
    let range_start = center_x.min(knob_x).saturating_sub(1);
    let range_end = center_x.max(knob_x).saturating_add(1).min(right);
    for y in center_y.saturating_sub(2)..=center_y + 2 {
        for x in range_start..=range_end {
            set_pixel(buffer, width, x, y, color);
        }
    }
}

fn draw_slider_knob(buffer: &mut [u32], width: usize, x: usize, top: usize, color: u32) {
    let knob_w = 16;
    let knob_h = SLIDER_HEIGHT.saturating_sub(10);
    let left = x.saturating_sub(knob_w / 2);
    let knob_top = top + 3;

    fill_rect(buffer, width, left, knob_top, knob_w, knob_h, color);
    draw_rect(buffer, width, left, knob_top, knob_w, knob_h, 0xFFFFFF);

    for y in knob_top + 6..knob_top + knob_h.saturating_sub(5) {
        set_pixel(buffer, width, x, y, 0xFFFFFF);
    }
}

fn draw_header(buffer: &mut [u32], width: usize) {
    let left = CANVAS_MARGIN;
    let right = width.saturating_sub(CANVAS_MARGIN);
    let band_width = right.saturating_sub(left);
    fill_rect(
        buffer,
        width,
        left,
        HEADER_TOP,
        band_width,
        HEADER_HEIGHT,
        0xFFFDF8,
    );
    draw_rect(
        buffer,
        width,
        left,
        HEADER_TOP,
        band_width,
        HEADER_HEIGHT,
        0xCCBBA4,
    );

    let accent_h = HEADER_HEIGHT.saturating_sub(16);
    fill_rect(buffer, width, left + 12, HEADER_TOP + 8, 220, accent_h, 0xF05C4B);
    fill_rect(buffer, width, left + 240, HEADER_TOP + 8, 160, accent_h, 0xF7AE3D);
    fill_rect(buffer, width, right.saturating_sub(210), HEADER_TOP + 8, 94, accent_h, 0x4E78D5);
    fill_rect(buffer, width, right.saturating_sub(108), HEADER_TOP + 8, 82, accent_h, 0x1B1F26);
    draw_text(
        buffer,
        width,
        left + 14,
        HEADER_TOP + 24,
        "LITE-ROOM PREVIEW",
        0xFFFFFF,
    );
}

fn fill_rect(buffer: &mut [u32], width: usize, left: usize, top: usize, w: usize, h: usize, color: u32) {
    for y in top..top.saturating_add(h) {
        for x in left..left.saturating_add(w) {
            set_pixel(buffer, width, x, y, color);
        }
    }
}

fn draw_rect(buffer: &mut [u32], width: usize, left: usize, top: usize, w: usize, h: usize, color: u32) {
    if w == 0 || h == 0 {
        return;
    }
    let right = left + w - 1;
    let bottom = top + h - 1;
    for x in left..=right {
        set_pixel(buffer, width, x, top, color);
        set_pixel(buffer, width, x, bottom, color);
    }
    for y in top..=bottom {
        set_pixel(buffer, width, left, y, color);
        set_pixel(buffer, width, right, y, color);
    }
}

fn lerp_color(start: u32, end: u32, t: f32) -> u32 {
    let clamped = t.clamp(0.0, 1.0);
    let sr = ((start >> 16) & 0xFF) as f32;
    let sg = ((start >> 8) & 0xFF) as f32;
    let sb = (start & 0xFF) as f32;
    let er = ((end >> 16) & 0xFF) as f32;
    let eg = ((end >> 8) & 0xFF) as f32;
    let eb = (end & 0xFF) as f32;

    let r = (sr + (er - sr) * clamped).round() as u32;
    let g = (sg + (eg - sg) * clamped).round() as u32;
    let b = (sb + (eb - sb) * clamped).round() as u32;
    (r << 16) | (g << 8) | b
}

fn darken_color(color: u32, amount: u8) -> u32 {
    let r = ((color >> 16) & 0xFF).saturating_sub(amount as u32);
    let g = ((color >> 8) & 0xFF).saturating_sub(amount as u32);
    let b = (color & 0xFF).saturating_sub(amount as u32);
    (r << 16) | (g << 8) | b
}

fn lighten_color(color: u32, amount: u8) -> u32 {
    let r = ((color >> 16) & 0xFF).saturating_add(amount as u32).min(255);
    let g = ((color >> 8) & 0xFF).saturating_add(amount as u32).min(255);
    let b = (color & 0xFF).saturating_add(amount as u32).min(255);
    (r << 16) | (g << 8) | b
}

fn draw_control_panel(buffer: &mut [u32], width: usize, height: usize) {
    let left = control_panel_left(width);
    let top = control_panel_top();
    let right = control_panel_right(width);
    let bottom = control_panel_bottom(height);
    let panel_w = right.saturating_sub(left);
    let panel_h = bottom.saturating_sub(top);

    fill_rect(buffer, width, left, top, panel_w, panel_h, 0xFBFAF7);
    draw_rect(buffer, width, left, top, panel_w, panel_h, 0xCCBBA4);

    let band_w = panel_w.saturating_sub(36);
    fill_rect(buffer, width, left + 18, top + 18, band_w, 16, 0x1A1F29);
    fill_rect(buffer, width, left + 18, top + 44, band_w, 46, 0xF0E3D0);
    draw_rect(buffer, width, left + 18, top + 44, band_w, 46, 0xD4C1A6);
}

fn draw_preview_shadow(buffer: &mut [u32], width: usize, height: usize) {
    let panel_left = preview_panel_left();
    let panel_top = preview_panel_top();
    let panel_right = preview_panel_right(width);
    let panel_bottom = preview_panel_bottom(height);

    for y in panel_top + 4..panel_bottom + 8 {
        for x in panel_left + 4..panel_right + 8 {
            let old = buffer[y * width + x];
            buffer[y * width + x] = darken_color(old, 14);
        }
    }
}

fn draw_slider_hover(buffer: &mut [u32], width: usize, field: SliderField, sliders: &[SliderSpec]) {
    if let Some(spec) = sliders.iter().find(|spec| spec.field == field) {
        let left = slider_left(width);
        let right = slider_right(width);
        draw_rect(
            buffer,
            width,
            left,
            spec.top.saturating_sub(1),
            right.saturating_sub(left).saturating_add(1),
            SLIDER_HEIGHT + 2,
            0x5A667A,
        );
    }
}

fn slider_left(width: usize) -> usize {
    control_panel_left(width).saturating_add(CONTROL_INSET)
}

fn slider_right(width: usize) -> usize {
    control_panel_right(width).saturating_sub(CONTROL_INSET)
}

fn preview_panel_left() -> usize {
    CANVAS_MARGIN
}

fn preview_panel_top() -> usize {
    WORKAREA_TOP
}

fn preview_panel_right(width: usize) -> usize {
    width.saturating_sub(CANVAS_MARGIN + CONTROL_PANEL_WIDTH + SPLIT_GUTTER)
}

fn preview_panel_bottom(height: usize) -> usize {
    height.saturating_sub(WORKAREA_BOTTOM_MARGIN)
}

fn control_panel_left(width: usize) -> usize {
    preview_panel_right(width).saturating_add(SPLIT_GUTTER)
}

fn control_panel_right(width: usize) -> usize {
    width.saturating_sub(CANVAS_MARGIN)
}

fn control_panel_top() -> usize {
    WORKAREA_TOP
}

fn control_panel_bottom(height: usize) -> usize {
    height.saturating_sub(WORKAREA_BOTTOM_MARGIN)
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
        .find(|spec| y >= spec.top.saturating_sub(2) && y <= spec.top + SLIDER_HEIGHT + 2)
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

fn draw_text(buffer: &mut [u32], width: usize, x: usize, y: usize, text: &str, color: u32) {
    let mut cursor_x = x;
    for ch in text.chars() {
        if ch == '\n' {
            continue;
        }
        draw_char(buffer, width, cursor_x, y, ch, color);
        cursor_x = cursor_x.saturating_add(8);
    }
}

fn draw_char(buffer: &mut [u32], width: usize, x: usize, y: usize, ch: char, color: u32) {
    let glyph = font8x8::BASIC_FONTS.get(ch).unwrap_or([0; 8]);
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..8 {
            if (bits >> col) & 1 == 1 {
                set_pixel(buffer, width, x + col, y + row, color);
            }
        }
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

fn slider_label(field: SliderField) -> &'static str {
    match field {
        SliderField::Exposure => "EXPOSURE",
        SliderField::Contrast => "CONTRAST",
        SliderField::Temperature => "TEMPERATURE",
        SliderField::Tint => "TINT",
        SliderField::Highlights => "HIGHLIGHTS",
        SliderField::Shadows => "SHADOWS",
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
    telemetry: TitleTelemetry<'_>,
) -> String {
    let preview_info = match telemetry.latest_frame {
        Some(frame) => format!(
            "preview seq={} {}x{} {}ms",
            frame.sequence, frame.width, frame.height, frame.render_time_ms
        ),
        None => "preview pending".to_string(),
    };
    let p95_text = telemetry
        .metrics
        .p95_render_time_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let last_text = telemetry
        .metrics
        .last_render_time_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let metric_info = format!(
        "jobs s/c/x/d={}/{}/{}/{} last={}ms p95={}ms",
        telemetry.metrics.submitted_jobs,
        telemetry.metrics.completed_jobs,
        telemetry.metrics.canceled_jobs,
        telemetry.metrics.dropped_frames,
        last_text,
        p95_text
    );
    let canvas_info = telemetry
        .preview_canvas
        .map(|canvas| format!("canvas={}x{}", canvas.width, canvas.height))
        .unwrap_or_else(|| "canvas=none".to_string());
    let slider_help = telemetry
        .focused_slider
        .map(|field| format!("focus={} ({})", field_name(field), slider_effect(field)))
        .unwrap_or_else(|| "focus=none (hover or drag slider)".to_string());
    let nav_info = telemetry
        .image_index
        .map(|(current, total)| format!("image {}/{} | left/right switch", current, total))
        .unwrap_or_else(|| "image 0/0 | left/right switch".to_string());

    match image_id {
        Some(image_id) => format!(
            "lite-room | catalog={} | cache={} | images={} | {} | edit image={} | drag sliders | {} | {} | {} | {} | {} | esc quit",
            catalog_path,
            cache_dir,
            image_count,
            nav_info,
            image_id.get(),
            build_slider_status(params),
            preview_info,
            metric_info,
            canvas_info,
            slider_help
        ),
        None => format!(
            "lite-room | catalog={} | cache={} | images={} | {} | no image to edit | {} | {} | {} | {} | esc quit",
            catalog_path,
            cache_dir,
            image_count,
            nav_info,
            preview_info,
            metric_info,
            canvas_info,
            slider_help
        ),
    }
}

fn slider_effect(field: SliderField) -> &'static str {
    match field {
        SliderField::Exposure => "overall brightness",
        SliderField::Contrast => "light-dark separation",
        SliderField::Temperature => "warm to cool color balance",
        SliderField::Tint => "green to magenta balance",
        SliderField::Highlights => "bright area detail",
        SliderField::Shadows => "dark area detail",
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
        let width = WINDOW_WIDTH;
        let mouse_x = (slider_right(width).saturating_sub(8)) as f32;
        let changed = update_param_from_mouse(&mut params, SliderField::Exposure, mouse_x, width);
        assert!(changed);
        assert!(params.exposure > 0.0);
        assert_eq!(params.contrast, 0.0);
    }
}
