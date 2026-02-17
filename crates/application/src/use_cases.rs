use lite_room_domain::ImageId;

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
