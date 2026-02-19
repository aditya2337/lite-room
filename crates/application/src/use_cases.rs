use lite_room_domain::EditParams;
use lite_room_domain::ImageId;
use lite_room_domain::PreviewRequest;

#[derive(Debug, Clone, Default)]
pub struct BootstrapCatalogCommand;

#[derive(Debug, Clone)]
pub struct ImportFolderCommand {
    pub folder: String,
    pub cache_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct ListImagesCommand;

#[derive(Debug, Clone, Copy)]
pub struct OpenImageCommand {
    pub image_id: ImageId,
}

#[derive(Debug, Clone, Copy)]
pub struct ShowEditCommand {
    pub image_id: ImageId,
}

#[derive(Debug, Clone, Copy)]
pub struct SetEditCommand {
    pub image_id: ImageId,
    pub params: EditParams,
}

#[derive(Debug, Clone, Copy)]
pub struct SubmitPreviewCommand {
    pub request: PreviewRequest,
}

#[derive(Debug, Clone, Default)]
pub struct PollPreviewCommand;

#[derive(Debug, Clone, Default)]
pub struct PreviewMetricsQuery;
