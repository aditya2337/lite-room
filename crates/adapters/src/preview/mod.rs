use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;

use image::io::Reader as ImageReader;
use lite_room_application::{ApplicationError, PreviewPipeline};
use lite_room_domain::{PreviewFrame, PreviewMetrics, PreviewRequest};
use wgpu::util::DeviceExt;

const METRIC_WINDOW_SIZE: usize = 64;
const MAX_RENDER_PIXELS: usize = 2_000_000;
const PREVIEW_WORKGROUP_SIZE: u32 = 64;
const PREVIEW_SHADER: &str = r#"
struct Params {
    pixel_count: u32,
    width: u32,
    exposure: f32,
    contrast: f32,
    temperature: f32,
    tint: f32,
    highlights: f32,
    shadows: f32,
}

@group(0) @binding(0)
var<storage, read> source_pixels: array<u32>;

@group(0) @binding(1)
var<storage, read_write> output_pixels: array<u32>;

@group(0) @binding(2)
var<uniform> params: Params;

fn to_u8(value: f32) -> u32 {
    return u32(clamp(value * 255.0, 0.0, 255.0));
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.pixel_count) {
        return;
    }

    let width = max(params.width, 1u);
    let source = source_pixels[i];
    var red = f32((source >> 16u) & 255u) / 255.0;
    var green = f32((source >> 8u) & 255u) / 255.0;
    var blue = f32(source & 255u) / 255.0;

    let exposure_gain = exp2(clamp(params.exposure, -5.0, 5.0));
    let contrast_factor = 1.0 + clamp(params.contrast, -5.0, 5.0) * 0.12;

    red = clamp((red * exposure_gain - 0.5) * contrast_factor + 0.5, 0.0, 1.0);
    green = clamp((green * exposure_gain - 0.5) * contrast_factor + 0.5, 0.0, 1.0);
    blue = clamp((blue * exposure_gain - 0.5) * contrast_factor + 0.5, 0.0, 1.0);

    let temp = clamp(params.temperature, -5.0, 5.0) * 0.035;
    let tint = clamp(params.tint, -5.0, 5.0) * 0.035;
    red = clamp(red + temp, 0.0, 1.0);
    blue = clamp(blue - temp, 0.0, 1.0);
    green = clamp(green + tint, 0.0, 1.0);

    let highlights = clamp(params.highlights, -5.0, 5.0) * 0.08;
    let shadows = clamp(params.shadows, -5.0, 5.0) * 0.08;
    let high_component = max(red - 0.5, 0.0) * highlights;
    let shadow_component = max(0.5 - red, 0.0) * shadows;
    red = clamp(red + shadow_component - high_component, 0.0, 1.0);

    let high_component_g = max(green - 0.5, 0.0) * highlights;
    let shadow_component_g = max(0.5 - green, 0.0) * shadows;
    green = clamp(green + shadow_component_g - high_component_g, 0.0, 1.0);

    let high_component_b = max(blue - 0.5, 0.0) * highlights;
    let shadow_component_b = max(0.5 - blue, 0.0) * shadows;
    blue = clamp(blue + shadow_component_b - high_component_b, 0.0, 1.0);

    let r = to_u8(red);
    let g = to_u8(green);
    let b = to_u8(blue);
    output_pixels[i] = (r << 16u) | (g << 8u) | b;
}
"#;

#[derive(Default)]
struct MetricsState {
    submitted_jobs: u64,
    completed_jobs: u64,
    canceled_jobs: u64,
    dropped_frames: u64,
    last_render_time_ms: Option<u64>,
    render_samples_ms: Vec<u64>,
}

impl MetricsState {
    fn snapshot(&self) -> PreviewMetrics {
        PreviewMetrics {
            submitted_jobs: self.submitted_jobs,
            completed_jobs: self.completed_jobs,
            canceled_jobs: self.canceled_jobs,
            dropped_frames: self.dropped_frames,
            last_render_time_ms: self.last_render_time_ms,
            p95_render_time_ms: percentile_95(&self.render_samples_ms),
        }
    }

    fn push_render_sample(&mut self, sample_ms: u64) {
        self.last_render_time_ms = Some(sample_ms);
        self.render_samples_ms.push(sample_ms);
        if self.render_samples_ms.len() > METRIC_WINDOW_SIZE {
            let drain_count = self.render_samples_ms.len() - METRIC_WINDOW_SIZE;
            self.render_samples_ms.drain(0..drain_count);
        }
    }
}

fn percentile_95(samples: &[u64]) -> Option<u64> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let index = (((sorted.len() - 1) as f64) * 0.95).round() as usize;
    sorted.get(index).copied()
}

#[derive(Debug, Clone)]
struct ScheduledJob {
    sequence: u64,
    request: PreviewRequest,
}

trait PreviewRenderer: Send + Sync {
    fn render(&self, request: PreviewRequest) -> Result<RenderedPreview, ApplicationError>;
}

struct RenderedPreview {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

struct WgpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl WgpuRenderer {
    fn new() -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| "no suitable wgpu adapter found".to_string())?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("lite-room-preview-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        ))
        .map_err(|error| format!("failed to create wgpu device: {error}"))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lite-room-preview-shader"),
            source: wgpu::ShaderSource::Wgsl(PREVIEW_SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lite-room-preview-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lite-room-preview-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("lite-room-preview-compute-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Ok(Self {
            device,
            queue,
            bind_group_layout,
            pipeline,
        })
    }
}

impl PreviewRenderer for WgpuRenderer {
    fn render(&self, request: PreviewRequest) -> Result<RenderedPreview, ApplicationError> {
        let width = request.target_width as usize;
        let height = request.target_height as usize;
        if width == 0 || height == 0 {
            return Err(ApplicationError::InvalidInput(
                "preview target dimensions must be non-zero".to_string(),
            ));
        }

        let (render_width, render_height, pixel_count) = render_target(width, height)?;
        let pixel_bytes = (pixel_count as u64) * 4;

        let source_pixels = decode_source_pixels(&request.source_path, render_width, render_height)?;
        let source_bytes = source_pixels_as_le_bytes(&source_pixels);
        let source = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("lite-room-preview-source"),
                contents: &source_bytes,
                usage: wgpu::BufferUsages::STORAGE,
            });

        let output = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lite-room-preview-output"),
            size: pixel_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let params = pack_gpu_params(request, render_width as u32, pixel_count as u32);
        let params_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("lite-room-preview-params"),
                contents: &params,
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lite-room-preview-readback"),
            size: pixel_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lite-room-preview-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: source.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("lite-room-preview-encoder"),
            });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("lite-room-preview-pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            let workgroups = ((pixel_count as u32) + PREVIEW_WORKGROUP_SIZE - 1) / PREVIEW_WORKGROUP_SIZE;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, pixel_bytes);
        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = readback.slice(..);
        let (tx, rx) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .map_err(|error| ApplicationError::Io(format!("gpu map channel failed: {error}")))?
            .map_err(|error| ApplicationError::Io(format!("gpu readback map failed: {error}")))?;

        let data = slice.get_mapped_range();
        let pixels = data
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect::<Vec<_>>();
        black_box_bytes(&data);
        drop(data);
        readback.unmap();
        Ok(RenderedPreview {
            width: render_width as u32,
            height: render_height as u32,
            pixels,
        })
    }
}

#[derive(Default)]
struct CpuStageRenderer;

impl PreviewRenderer for CpuStageRenderer {
    fn render(&self, request: PreviewRequest) -> Result<RenderedPreview, ApplicationError> {
        let width = request.target_width as usize;
        let height = request.target_height as usize;
        if width == 0 || height == 0 {
            return Err(ApplicationError::InvalidInput(
                "preview target dimensions must be non-zero".to_string(),
            ));
        }

        let (render_width, render_height, _) = render_target(width, height)?;
        let mut pixels = decode_source_pixels(&request.source_path, render_width, render_height)?;
        apply_exposure_contrast(&mut pixels, request.params.exposure, request.params.contrast);
        apply_temperature_tint(&mut pixels, request.params.temperature, request.params.tint);
        apply_highlights_shadows(&mut pixels, request.params.highlights, request.params.shadows);
        black_box_checksum(&pixels);
        Ok(RenderedPreview {
            width: render_width as u32,
            height: render_height as u32,
            pixels,
        })
    }
}

pub struct BackgroundPreviewPipeline {
    next_sequence: AtomicU64,
    latest_sequence: Arc<AtomicU64>,
    submit_tx: mpsc::Sender<ScheduledJob>,
    result_rx: Mutex<mpsc::Receiver<PreviewFrame>>,
    metrics: Arc<Mutex<MetricsState>>,
    _renderer: Arc<dyn PreviewRenderer>,
}

impl BackgroundPreviewPipeline {
    pub fn new() -> Self {
        let renderer: Arc<dyn PreviewRenderer> = match WgpuRenderer::new() {
            Ok(renderer) => Arc::new(renderer),
            Err(_) => Arc::new(CpuStageRenderer),
        };
        Self::with_renderer(renderer)
    }

    fn with_renderer(renderer: Arc<dyn PreviewRenderer>) -> Self {
        let (submit_tx, submit_rx) = mpsc::channel::<ScheduledJob>();
        let (result_tx, result_rx) = mpsc::channel::<PreviewFrame>();
        let latest_sequence = Arc::new(AtomicU64::new(0));
        let metrics = Arc::new(Mutex::new(MetricsState::default()));

        spawn_worker(
            submit_rx,
            result_tx,
            Arc::clone(&latest_sequence),
            Arc::clone(&metrics),
            Arc::clone(&renderer),
        );

        Self {
            next_sequence: AtomicU64::new(0),
            latest_sequence,
            submit_tx,
            result_rx: Mutex::new(result_rx),
            metrics,
            _renderer: renderer,
        }
    }
}

impl Default for BackgroundPreviewPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewPipeline for BackgroundPreviewPipeline {
    fn submit_preview(&self, request: PreviewRequest) -> Result<(), ApplicationError> {
        let sequence = self.next_sequence.fetch_add(1, Ordering::SeqCst) + 1;
        self.latest_sequence.store(sequence, Ordering::SeqCst);
        {
            let mut metrics = self
                .metrics
                .lock()
                .map_err(|_| ApplicationError::Io("preview metrics lock poisoned".to_string()))?;
            metrics.submitted_jobs += 1;
        }
        self.submit_tx
            .send(ScheduledJob { sequence, request })
            .map_err(|error| ApplicationError::Io(format!("failed to enqueue preview job: {error}")))
    }

    fn try_receive_preview(&self) -> Result<Option<PreviewFrame>, ApplicationError> {
        let receiver = self
            .result_rx
            .lock()
            .map_err(|_| ApplicationError::Io("preview result lock poisoned".to_string()))?;

        let first = match receiver.try_recv() {
            Ok(frame) => frame,
            Err(mpsc::TryRecvError::Empty) => return Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err(ApplicationError::Io(
                    "preview result channel disconnected".to_string(),
                ))
            }
        };

        let mut newest = first;
        let mut dropped = 0_u64;
        while let Ok(next) = receiver.try_recv() {
            dropped += 1;
            newest = next;
        }

        if dropped > 0 {
            let mut metrics = self
                .metrics
                .lock()
                .map_err(|_| ApplicationError::Io("preview metrics lock poisoned".to_string()))?;
            metrics.dropped_frames += dropped;
        }

        Ok(Some(newest))
    }

    fn metrics(&self) -> Result<PreviewMetrics, ApplicationError> {
        let metrics = self
            .metrics
            .lock()
            .map_err(|_| ApplicationError::Io("preview metrics lock poisoned".to_string()))?;
        Ok(metrics.snapshot())
    }
}

fn spawn_worker(
    submit_rx: mpsc::Receiver<ScheduledJob>,
    result_tx: mpsc::Sender<PreviewFrame>,
    latest_sequence: Arc<AtomicU64>,
    metrics: Arc<Mutex<MetricsState>>,
    renderer: Arc<dyn PreviewRenderer>,
) {
    thread::spawn(move || {
        while let Ok(mut job) = submit_rx.recv() {
            while let Ok(next) = submit_rx.try_recv() {
                mark_canceled(&metrics, 1);
                job = next;
            }

            if job.sequence < latest_sequence.load(Ordering::SeqCst) {
                mark_canceled(&metrics, 1);
                continue;
            }

            let image_id = job.request.image_id;
            let started = Instant::now();
            let rendered = match renderer.render(job.request) {
                Ok(rendered) => rendered,
                Err(_) => {
                mark_canceled(&metrics, 1);
                continue;
                }
            };
            let elapsed = started.elapsed().as_millis() as u64;

            if job.sequence < latest_sequence.load(Ordering::SeqCst) {
                mark_canceled(&metrics, 1);
                continue;
            }

            let frame = PreviewFrame {
                image_id,
                sequence: job.sequence,
                width: rendered.width,
                height: rendered.height,
                render_time_ms: elapsed,
                pixels: rendered.pixels,
            };
            if result_tx.send(frame).is_err() {
                return;
            }

            if let Ok(mut m) = metrics.lock() {
                m.completed_jobs += 1;
                m.push_render_sample(elapsed);
            }
        }
    });
}

fn mark_canceled(metrics: &Arc<Mutex<MetricsState>>, count: u64) {
    if let Ok(mut m) = metrics.lock() {
        m.canceled_jobs += count;
    }
}

fn decode_source_pixels(
    source_path: &str,
    target_width: usize,
    target_height: usize,
) -> Result<Vec<u32>, ApplicationError> {
    let image = ImageReader::open(source_path)
        .map_err(|error| ApplicationError::Decode(error.to_string()))?
        .with_guessed_format()
        .map_err(|error| ApplicationError::Decode(error.to_string()))?
        .decode()
        .map_err(|error| ApplicationError::Decode(error.to_string()))?;
    let source = image.to_rgb8();
    let src_width = source.width() as usize;
    let src_height = source.height() as usize;
    if src_width == 0 || src_height == 0 {
        return Err(ApplicationError::Decode(format!(
            "empty image dimensions for source path: {}",
            source_path
        )));
    }

    let mut pixels = vec![0_u32; target_width * target_height];
    for y in 0..target_height {
        let src_y = y * src_height / target_height;
        for x in 0..target_width {
            let src_x = x * src_width / target_width;
            let pixel = source.get_pixel(src_x as u32, src_y as u32);
            let [red, green, blue] = pixel.0;
            pixels[y * target_width + x] =
                ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32);
        }
    }
    Ok(pixels)
}

fn source_pixels_as_le_bytes(pixels: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(pixels.len() * 4);
    for pixel in pixels {
        bytes.extend_from_slice(&pixel.to_le_bytes());
    }
    bytes
}

fn apply_exposure_contrast(pixels: &mut [u32], exposure: f32, contrast: f32) {
    let exposure_gain = 2_f32.powf(exposure.clamp(-5.0, 5.0));
    let contrast_factor = 1.0 + contrast.clamp(-5.0, 5.0) * 0.12;

    for pixel in pixels.iter_mut() {
        let [mut r, mut g, mut b] = unpack_rgb(*pixel);
        r = apply_exposure_and_contrast_channel(r, exposure_gain, contrast_factor);
        g = apply_exposure_and_contrast_channel(g, exposure_gain, contrast_factor);
        b = apply_exposure_and_contrast_channel(b, exposure_gain, contrast_factor);
        *pixel = pack_rgb(r, g, b);
    }
}

fn apply_temperature_tint(pixels: &mut [u32], temperature: f32, tint: f32) {
    let temp = temperature.clamp(-5.0, 5.0) * 0.035;
    let tint_shift = tint.clamp(-5.0, 5.0) * 0.035;

    for pixel in pixels.iter_mut() {
        let [r, g, b] = unpack_rgb(*pixel);
        let red = (r as f32 / 255.0 + temp).clamp(0.0, 1.0);
        let blue = (b as f32 / 255.0 - temp).clamp(0.0, 1.0);
        let green = (g as f32 / 255.0 + tint_shift).clamp(0.0, 1.0);
        *pixel = pack_rgb(
            (red * 255.0).round() as u8,
            (green * 255.0).round() as u8,
            (blue * 255.0).round() as u8,
        );
    }
}

fn apply_highlights_shadows(pixels: &mut [u32], highlights: f32, shadows: f32) {
    let highlights_strength = highlights.clamp(-5.0, 5.0) * 0.08;
    let shadows_strength = shadows.clamp(-5.0, 5.0) * 0.08;

    for pixel in pixels.iter_mut() {
        let [r, g, b] = unpack_rgb(*pixel);
        *pixel = pack_rgb(
            apply_highlights_shadows_channel(r, highlights_strength, shadows_strength),
            apply_highlights_shadows_channel(g, highlights_strength, shadows_strength),
            apply_highlights_shadows_channel(b, highlights_strength, shadows_strength),
        );
    }
}

fn black_box_checksum(pixels: &[u32]) {
    let checksum = pixels
        .iter()
        .fold(0_u64, |acc, value| acc.wrapping_add(u64::from(*value)));
    std::hint::black_box(checksum);
}

fn black_box_bytes(bytes: &[u8]) {
    let checksum = bytes
        .iter()
        .fold(0_u64, |acc, value| acc.wrapping_add(u64::from(*value)));
    std::hint::black_box(checksum);
}

fn pack_gpu_params(request: PreviewRequest, render_width: u32, pixel_count: u32) -> [u8; 32] {
    let mut out = [0_u8; 32];
    out[0..4].copy_from_slice(&pixel_count.to_le_bytes());
    out[4..8].copy_from_slice(&render_width.to_le_bytes());
    out[8..12].copy_from_slice(&request.params.exposure.to_le_bytes());
    out[12..16].copy_from_slice(&request.params.contrast.to_le_bytes());
    out[16..20].copy_from_slice(&request.params.temperature.to_le_bytes());
    out[20..24].copy_from_slice(&request.params.tint.to_le_bytes());
    out[24..28].copy_from_slice(&request.params.highlights.to_le_bytes());
    out[28..32].copy_from_slice(&request.params.shadows.to_le_bytes());
    out
}

fn render_target(width: usize, height: usize) -> Result<(usize, usize, usize), ApplicationError> {
    let requested_pixels = width
        .checked_mul(height)
        .ok_or_else(|| ApplicationError::InvalidInput("preview dimensions overflow".to_string()))?;
    if requested_pixels <= MAX_RENDER_PIXELS {
        return Ok((width, height, requested_pixels));
    }

    let scale = (MAX_RENDER_PIXELS as f64 / requested_pixels as f64).sqrt();
    let render_width = ((width as f64 * scale).floor() as usize).max(1);
    let render_height = ((height as f64 * scale).floor() as usize).max(1);
    let pixel_count = render_width
        .checked_mul(render_height)
        .ok_or_else(|| ApplicationError::InvalidInput("preview dimensions overflow".to_string()))?;
    Ok((render_width, render_height, pixel_count.min(MAX_RENDER_PIXELS)))
}

fn unpack_rgb(pixel: u32) -> [u8; 3] {
    [
        ((pixel >> 16) & 0xFF) as u8,
        ((pixel >> 8) & 0xFF) as u8,
        (pixel & 0xFF) as u8,
    ]
}

fn pack_rgb(red: u8, green: u8, blue: u8) -> u32 {
    ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32)
}

fn apply_exposure_and_contrast_channel(channel: u8, exposure_gain: f32, contrast_factor: f32) -> u8 {
    let normalized = channel as f32 / 255.0;
    let exposed = normalized * exposure_gain;
    let contrasted = ((exposed - 0.5) * contrast_factor + 0.5).clamp(0.0, 1.0);
    (contrasted * 255.0).round() as u8
}

fn apply_highlights_shadows_channel(channel: u8, highlights_strength: f32, shadows_strength: f32) -> u8 {
    let value = channel as f32 / 255.0;
    let highlight_component = (value - 0.5).max(0.0) * highlights_strength;
    let shadow_component = (0.5 - value).max(0.0) * shadows_strength;
    let adjusted = (value + shadow_component - highlight_component).clamp(0.0, 1.0);
    (adjusted * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use lite_room_domain::{EditParams, ImageId};
    use tempfile::tempdir;
    use std::time::{Duration, Instant};

    fn write_test_jpeg(dir: &tempfile::TempDir) -> String {
        let path = dir.path().join("preview.jpg");
        let pixels = ImageBuffer::from_pixel(8, 8, Rgb([120_u8, 80_u8, 40_u8]));
        pixels.save(&path).expect("save jpeg");
        path.to_string_lossy().to_string()
    }

    #[test]
    fn latest_job_wins_and_old_jobs_cancel() {
        let pipeline = BackgroundPreviewPipeline::new();
        let image_id = ImageId::new(1).expect("id");
        let temp = tempdir().expect("tempdir");
        let source_path = write_test_jpeg(&temp);

        for i in 0..8 {
            let params = EditParams {
                exposure: i as f32,
                ..EditParams::default()
            };
            pipeline
                .submit_preview(PreviewRequest {
                    image_id,
                    source_path: source_path.clone(),
                    params,
                    target_width: 1200,
                    target_height: 800,
                })
                .expect("submit preview");
        }

        let deadline = Instant::now() + Duration::from_millis(600);
        let frame = loop {
            if let Some(frame) = pipeline.try_receive_preview().expect("poll") {
                break frame;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for preview frame"
            );
            thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(frame.sequence, 8);

        let metrics = pipeline.metrics().expect("metrics");
        assert!(metrics.canceled_jobs >= 1);
        assert_eq!(metrics.completed_jobs, 1);
    }

    #[test]
    fn renderer_rejects_zero_dimensions() {
        let renderer = CpuStageRenderer;
        let image_id = ImageId::new(1).expect("id");
        let result = renderer.render(PreviewRequest {
            image_id,
            source_path: "ignored.jpg".to_string(),
            params: EditParams::default(),
            target_width: 0,
            target_height: 512,
        });

        assert!(matches!(result, Err(ApplicationError::InvalidInput(_))));
    }
}
