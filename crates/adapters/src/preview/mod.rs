use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use lite_room_application::{ApplicationError, PreviewPipeline};
use lite_room_domain::{PreviewFrame, PreviewMetrics, PreviewRequest};

const METRIC_WINDOW_SIZE: usize = 64;

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

#[derive(Debug, Clone, Copy)]
struct ScheduledJob {
    sequence: u64,
    request: PreviewRequest,
}

pub struct BackgroundPreviewPipeline {
    next_sequence: AtomicU64,
    latest_sequence: Arc<AtomicU64>,
    submit_tx: mpsc::Sender<ScheduledJob>,
    result_rx: Mutex<mpsc::Receiver<PreviewFrame>>,
    metrics: Arc<Mutex<MetricsState>>,
}

impl BackgroundPreviewPipeline {
    pub fn new() -> Self {
        let (submit_tx, submit_rx) = mpsc::channel::<ScheduledJob>();
        let (result_tx, result_rx) = mpsc::channel::<PreviewFrame>();
        let latest_sequence = Arc::new(AtomicU64::new(0));
        let metrics = Arc::new(Mutex::new(MetricsState::default()));

        spawn_worker(submit_rx, result_tx, Arc::clone(&latest_sequence), Arc::clone(&metrics));

        Self {
            next_sequence: AtomicU64::new(0),
            latest_sequence,
            submit_tx,
            result_rx: Mutex::new(result_rx),
            metrics,
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

            let started = Instant::now();
            thread::sleep(simulated_render_duration(job.request));
            let elapsed = started.elapsed().as_millis() as u64;

            if job.sequence < latest_sequence.load(Ordering::SeqCst) {
                mark_canceled(&metrics, 1);
                continue;
            }

            let frame = PreviewFrame {
                image_id: job.request.image_id,
                sequence: job.sequence,
                width: job.request.target_width,
                height: job.request.target_height,
                render_time_ms: elapsed,
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

fn simulated_render_duration(request: PreviewRequest) -> Duration {
    let pixels = u64::from(request.target_width) * u64::from(request.target_height);
    let budget = pixels / 150_000;
    let base = 8_u64;
    Duration::from_millis((base + budget).min(40))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lite_room_domain::{EditParams, ImageId};

    #[test]
    fn latest_job_wins_and_old_jobs_cancel() {
        let pipeline = BackgroundPreviewPipeline::new();
        let image_id = ImageId::new(1).expect("id");

        for i in 0..8 {
            let params = EditParams {
                exposure: i as f32,
                ..EditParams::default()
            };
            pipeline
                .submit_preview(PreviewRequest {
                    image_id,
                    params,
                    target_width: 1200,
                    target_height: 800,
                })
                .expect("submit preview");
        }

        thread::sleep(Duration::from_millis(120));
        let frame = pipeline
            .try_receive_preview()
            .expect("poll")
            .expect("frame");
        assert_eq!(frame.sequence, 8);

        let metrics = pipeline.metrics().expect("metrics");
        assert!(metrics.canceled_jobs >= 1);
        assert_eq!(metrics.completed_jobs, 1);
    }
}

