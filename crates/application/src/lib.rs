mod error;
mod ports;
mod service;
mod use_cases;

pub use error::ApplicationError;
pub use ports::{
    CatalogRepository, Clock, FileScanSummary, FileScanner, ImageDecoder, NewImage,
    PreviewPipeline, ScannedFile, StoredEdit, ThumbnailArtifact, ThumbnailGenerator,
    UpsertImageResult,
};
pub use service::ApplicationService;
pub use use_cases::{
    BootstrapCatalogCommand, ImportFolderCommand, ListImagesCommand, OpenImageCommand,
    PollPreviewCommand, PreviewMetricsQuery, SetEditCommand, ShowEditCommand,
    SubmitPreviewCommand,
};
