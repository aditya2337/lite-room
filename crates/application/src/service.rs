use lite_room_domain::{
    DecodedImage, EditParams, ImageRecord, ImportReport, PreviewFrame, PreviewMetrics,
};
use serde_json::json;

use crate::{
    ApplicationError, BootstrapCatalogCommand, CatalogRepository, Clock, FileScanner,
    ImageDecoder, ImportFolderCommand, ListImagesCommand, OpenImageCommand, PollPreviewCommand,
    PreviewMetricsQuery, PreviewPipeline, SetEditCommand, ShowEditCommand, SubmitPreviewCommand,
    ThumbnailGenerator,
};

pub struct ApplicationService {
    catalog: Box<dyn CatalogRepository>,
    scanner: Box<dyn FileScanner>,
    thumbnails: Box<dyn ThumbnailGenerator>,
    decoder: Box<dyn ImageDecoder>,
    clock: Box<dyn Clock>,
    preview: Box<dyn PreviewPipeline>,
}

impl ApplicationService {
    pub fn new(
        catalog: Box<dyn CatalogRepository>,
        scanner: Box<dyn FileScanner>,
        thumbnails: Box<dyn ThumbnailGenerator>,
        decoder: Box<dyn ImageDecoder>,
        clock: Box<dyn Clock>,
        preview: Box<dyn PreviewPipeline>,
    ) -> Self {
        Self {
            catalog,
            scanner,
            thumbnails,
            decoder,
            clock,
            preview,
        }
    }

    pub fn bootstrap_catalog(
        &self,
        _command: BootstrapCatalogCommand,
    ) -> Result<(), ApplicationError> {
        self.catalog.initialize()
    }

    pub fn import_folder(
        &self,
        command: ImportFolderCommand,
    ) -> Result<ImportReport, ApplicationError> {
        if command.folder.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "folder path must not be empty".to_string(),
            ));
        }
        if command.cache_root.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "cache root must not be empty".to_string(),
            ));
        }

        let scan = self.scanner.scan_supported(&command.folder)?;
        let now = self.clock.now_timestamp_string();
        let edit = EditParams::default();
        edit.validate()?;
        let default_edit_json = serde_json::to_string(&edit)
            .map_err(|error| ApplicationError::Persistence(error.to_string()))?;

        let mut report = ImportReport {
            scanned_files: scan.scanned_files,
            supported_files: scan.supported_files,
            newly_imported: 0,
        };

        for file in scan.files {
            let metadata_json = json!({
                "file_size": file.file_size,
                "extension": file.extension,
            })
            .to_string();

            let upsert = self.catalog.upsert_image(&crate::NewImage {
                file_path: file.canonical_path.to_string_lossy().to_string(),
                import_date: now.clone(),
                capture_date: None,
                camera_model: None,
                iso: None,
                rating: 0,
                flag: 0,
                metadata_json,
            })?;

            if upsert.inserted {
                report.newly_imported += 1;
            }

            self.catalog
                .ensure_default_edit(upsert.image_id, &default_edit_json, &now)?;

            let thumb = self.thumbnails.ensure_thumbnail(
                &file.canonical_path,
                &command.cache_root,
                upsert.image_id,
            )?;

            self.catalog.upsert_thumbnail(
                upsert.image_id,
                &thumb.file_path,
                i64::from(thumb.width),
                i64::from(thumb.height),
                &now,
            )?;
        }

        Ok(report)
    }

    pub fn list_images(
        &self,
        _command: ListImagesCommand,
    ) -> Result<Vec<ImageRecord>, ApplicationError> {
        self.catalog.list_images()
    }

    pub fn open_image(&self, command: OpenImageCommand) -> Result<DecodedImage, ApplicationError> {
        let image = self
            .catalog
            .find_image_by_id(command.image_id)?
            .ok_or_else(|| {
                ApplicationError::NotFound(format!(
                    "image not found for id={}",
                    command.image_id.get()
                ))
            })?;
        self.decoder
            .decode_for_preview(std::path::Path::new(&image.file_path))
    }

    pub fn show_edit(&self, command: ShowEditCommand) -> Result<EditParams, ApplicationError> {
        self.catalog
            .find_edit(command.image_id)?
            .map(|stored| {
                serde_json::from_str::<EditParams>(&stored.edit_params_json)
                    .map_err(|error| ApplicationError::Persistence(error.to_string()))
            })
            .transpose()?
            .ok_or_else(|| {
                ApplicationError::NotFound(format!(
                    "edit not found for image id={}",
                    command.image_id.get()
                ))
            })
    }

    pub fn set_edit(&self, command: SetEditCommand) -> Result<(), ApplicationError> {
        command.params.validate()?;
        let now = self.clock.now_timestamp_string();
        let edit_json = serde_json::to_string(&command.params)
            .map_err(|error| ApplicationError::Persistence(error.to_string()))?;
        self.catalog
            .upsert_edit(command.image_id, &edit_json, &now)?;
        Ok(())
    }

    pub fn submit_preview(&self, command: SubmitPreviewCommand) -> Result<(), ApplicationError> {
        command.request.params.validate()?;
        self.preview.submit_preview(command.request)
    }

    pub fn poll_preview(
        &self,
        _command: PollPreviewCommand,
    ) -> Result<Option<PreviewFrame>, ApplicationError> {
        self.preview.try_receive_preview()
    }

    pub fn preview_metrics(
        &self,
        _query: PreviewMetricsQuery,
    ) -> Result<PreviewMetrics, ApplicationError> {
        self.preview.metrics()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use lite_room_domain::{detect_image_kind, DecodedImage, ImageId, ImageKind, ImageRecord};

    use super::*;

    struct FakeCatalog {
        initialized: std::cell::Cell<bool>,
        next_id: std::cell::Cell<i64>,
        images: std::cell::RefCell<HashMap<i64, ImageRecord>>,
        edits: std::cell::RefCell<HashMap<i64, crate::StoredEdit>>,
    }

    #[derive(Default)]
    struct FakePreviewPipeline {
        submitted: std::cell::RefCell<Vec<lite_room_domain::PreviewRequest>>,
        responses: std::cell::RefCell<Vec<lite_room_domain::PreviewFrame>>,
    }

    impl PreviewPipeline for FakePreviewPipeline {
        fn submit_preview(
            &self,
            request: lite_room_domain::PreviewRequest,
        ) -> Result<(), ApplicationError> {
            self.submitted.borrow_mut().push(request);
            Ok(())
        }

        fn try_receive_preview(
            &self,
        ) -> Result<Option<lite_room_domain::PreviewFrame>, ApplicationError> {
            Ok(self.responses.borrow_mut().pop())
        }

        fn metrics(&self) -> Result<lite_room_domain::PreviewMetrics, ApplicationError> {
            Ok(lite_room_domain::PreviewMetrics::default())
        }
    }

    impl FakeCatalog {
        fn new() -> Self {
            Self {
                initialized: std::cell::Cell::new(false),
                next_id: std::cell::Cell::new(1),
                images: std::cell::RefCell::new(HashMap::new()),
                edits: std::cell::RefCell::new(HashMap::new()),
            }
        }
    }

    impl CatalogRepository for FakeCatalog {
        fn initialize(&self) -> Result<(), ApplicationError> {
            self.initialized.set(true);
            Ok(())
        }

        fn upsert_image(
            &self,
            image: &crate::NewImage,
        ) -> Result<crate::UpsertImageResult, ApplicationError> {
            let mut images = self.images.borrow_mut();
            if let Some(found) = images
                .values()
                .find(|entry| entry.file_path == image.file_path)
            {
                return Ok(crate::UpsertImageResult {
                    image_id: found.id,
                    inserted: false,
                });
            }

            let id_value = self.next_id.get();
            self.next_id.set(id_value + 1);
            let image_id = ImageId::new(id_value).expect("positive id");
            images.insert(
                id_value,
                ImageRecord {
                    id: image_id,
                    file_path: image.file_path.clone(),
                    import_date: image.import_date.clone(),
                    capture_date: image.capture_date.clone(),
                    rating: image.rating,
                    flag: image.flag,
                    metadata_json: image.metadata_json.clone(),
                },
            );
            Ok(crate::UpsertImageResult {
                image_id,
                inserted: true,
            })
        }

        fn ensure_default_edit(
            &self,
            image_id: ImageId,
            edit_params_json: &str,
            updated_at: &str,
        ) -> Result<(), ApplicationError> {
            self.edits
                .borrow_mut()
                .entry(image_id.get())
                .or_insert_with(|| crate::StoredEdit {
                    edit_params_json: edit_params_json.to_string(),
                    updated_at: updated_at.to_string(),
                });
            Ok(())
        }

        fn upsert_edit(
            &self,
            image_id: ImageId,
            edit_params_json: &str,
            updated_at: &str,
        ) -> Result<(), ApplicationError> {
            self.edits.borrow_mut().insert(
                image_id.get(),
                crate::StoredEdit {
                    edit_params_json: edit_params_json.to_string(),
                    updated_at: updated_at.to_string(),
                },
            );
            Ok(())
        }

        fn find_edit(
            &self,
            image_id: ImageId,
        ) -> Result<Option<crate::StoredEdit>, ApplicationError> {
            Ok(self.edits.borrow().get(&image_id.get()).cloned())
        }

        fn upsert_thumbnail(
            &self,
            _image_id: ImageId,
            _file_path: &str,
            _width: i64,
            _height: i64,
            _updated_at: &str,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn list_images(&self) -> Result<Vec<ImageRecord>, ApplicationError> {
            Ok(self.images.borrow().values().cloned().collect())
        }

        fn find_image_by_id(
            &self,
            image_id: ImageId,
        ) -> Result<Option<ImageRecord>, ApplicationError> {
            Ok(self.images.borrow().get(&image_id.get()).cloned())
        }
    }

    struct FakeScanner {
        files: Vec<PathBuf>,
    }

    impl FileScanner for FakeScanner {
        fn scan_supported(
            &self,
            _folder: &str,
        ) -> Result<crate::FileScanSummary, ApplicationError> {
            let scanned_files = self.files.len();
            let files: Vec<crate::ScannedFile> = self
                .files
                .iter()
                .map(|path| {
                    let ext = path
                        .extension()
                        .and_then(|part| part.to_str())
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    crate::ScannedFile {
                        canonical_path: path.clone(),
                        extension: ext,
                        file_size: 100,
                        image_kind: detect_image_kind(path),
                    }
                })
                .collect();
            Ok(crate::FileScanSummary {
                scanned_files,
                supported_files: files.len(),
                files,
            })
        }
    }

    struct FakeThumbs;

    impl ThumbnailGenerator for FakeThumbs {
        fn ensure_thumbnail(
            &self,
            _source_path: &Path,
            cache_root: &str,
            image_id: ImageId,
        ) -> Result<crate::ThumbnailArtifact, ApplicationError> {
            Ok(crate::ThumbnailArtifact {
                file_path: format!("{cache_root}/thumbs/{}.jpg", image_id.get()),
                width: 256,
                height: 256,
            })
        }
    }

    struct FakeDecoder;

    impl ImageDecoder for FakeDecoder {
        fn decode_for_preview(&self, path: &Path) -> Result<DecodedImage, ApplicationError> {
            Ok(DecodedImage {
                width: 64,
                height: 48,
                kind: detect_image_kind(path),
            })
        }
    }

    struct FakeClock;

    impl Clock for FakeClock {
        fn now_timestamp_string(&self) -> String {
            "123".to_string()
        }
    }

    #[test]
    fn import_and_open_image_workflow() {
        let service = ApplicationService::new(
            Box::new(FakeCatalog::new()),
            Box::new(FakeScanner {
                files: vec![PathBuf::from("/tmp/sample.jpg")],
            }),
            Box::new(FakeThumbs),
            Box::new(FakeDecoder),
            Box::new(FakeClock),
            Box::<FakePreviewPipeline>::default(),
        );

        service
            .bootstrap_catalog(BootstrapCatalogCommand)
            .expect("bootstrap should work");

        let report = service
            .import_folder(ImportFolderCommand {
                folder: "/tmp".to_string(),
                cache_root: "cache".to_string(),
            })
            .expect("import should work");
        assert_eq!(report.scanned_files, 1);
        assert_eq!(report.supported_files, 1);
        assert_eq!(report.newly_imported, 1);

        let images = service
            .list_images(ListImagesCommand)
            .expect("list should work");
        assert_eq!(images.len(), 1);

        let decoded = service
            .open_image(OpenImageCommand {
                image_id: images[0].id,
            })
            .expect("open should work");
        assert_eq!(decoded.width, 64);
        assert_eq!(decoded.kind, ImageKind::Jpeg);
    }

    #[test]
    fn open_missing_image_returns_not_found() {
        let service = ApplicationService::new(
            Box::new(FakeCatalog::new()),
            Box::new(FakeScanner { files: vec![] }),
            Box::new(FakeThumbs),
            Box::new(FakeDecoder),
            Box::new(FakeClock),
            Box::<FakePreviewPipeline>::default(),
        );

        let result = service.open_image(OpenImageCommand {
            image_id: ImageId::new(99).expect("id"),
        });

        assert!(matches!(result, Err(ApplicationError::NotFound(_))));
    }

    #[test]
    fn set_and_show_edit_roundtrip() {
        let service = ApplicationService::new(
            Box::new(FakeCatalog::new()),
            Box::new(FakeScanner {
                files: vec![PathBuf::from("/tmp/sample.jpg")],
            }),
            Box::new(FakeThumbs),
            Box::new(FakeDecoder),
            Box::new(FakeClock),
            Box::<FakePreviewPipeline>::default(),
        );

        let report = service
            .import_folder(ImportFolderCommand {
                folder: "/tmp".to_string(),
                cache_root: "cache".to_string(),
            })
            .expect("import should work");
        assert_eq!(report.newly_imported, 1);

        let image = service
            .list_images(ListImagesCommand)
            .expect("list should work")
            .into_iter()
            .next()
            .expect("one image");

        let params = EditParams {
            exposure: 0.5,
            contrast: 0.1,
            temperature: -5.0,
            tint: 2.0,
            highlights: -10.0,
            shadows: 8.0,
        };

        service
            .set_edit(SetEditCommand {
                image_id: image.id,
                params,
            })
            .expect("set edit should work");

        let loaded = service
            .show_edit(ShowEditCommand { image_id: image.id })
            .expect("show edit should work");
        assert_eq!(loaded, params);
    }
}
