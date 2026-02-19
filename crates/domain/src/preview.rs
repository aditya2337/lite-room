use crate::{EditParams, ImageId};

#[derive(Debug, Clone, PartialEq)]
pub struct PreviewRequest {
    pub image_id: ImageId,
    pub source_path: String,
    pub params: EditParams,
    pub target_width: u32,
    pub target_height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewFrame {
    pub image_id: ImageId,
    pub sequence: u64,
    pub width: u32,
    pub height: u32,
    pub render_time_ms: u64,
    pub pixels: Vec<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreviewMetrics {
    pub submitted_jobs: u64,
    pub completed_jobs: u64,
    pub canceled_jobs: u64,
    pub dropped_frames: u64,
    pub last_render_time_ms: Option<u64>,
    pub p95_render_time_ms: Option<u64>,
}
