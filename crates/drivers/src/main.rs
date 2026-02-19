mod config;
mod logging;
mod ui;

use std::process::ExitCode;

use config::AppConfig;
use lite_room_adapters::{
    present_decoded, present_edit_params, present_image_row, BackgroundPreviewPipeline,
    FsThumbnailGenerator, ImageCrateDecoder, SqliteCatalogRepository, SystemClock,
    WalkdirFileScanner,
};
use lite_room_application::{
    ApplicationService, BootstrapCatalogCommand, ImportFolderCommand, ListImagesCommand,
    OpenImageCommand, SetEditCommand, ShowEditCommand,
};
use lite_room_domain::{EditParams, ImageId};

fn main() -> ExitCode {
    logging::init_logging();
    let args: Vec<String> = std::env::args().collect();
    let config = AppConfig::default();

    let service = build_application_service(&config);
    if let Err(error) = service.bootstrap_catalog(BootstrapCatalogCommand) {
        eprintln!("failed to bootstrap lite-room: {error}");
        return ExitCode::from(1);
    }

    let command = parse_command(&args);
    match run_command(command, &service, &config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CommandError::Usage(msg)) => {
            eprintln!("{msg}");
            print_usage();
            ExitCode::from(2)
        }
        Err(CommandError::Runtime(msg)) => {
            eprintln!("{msg}");
            ExitCode::from(1)
        }
    }
}

fn build_application_service(config: &AppConfig) -> ApplicationService {
    ApplicationService::new(
        Box::new(SqliteCatalogRepository::new(config.catalog_path.clone())),
        Box::new(WalkdirFileScanner),
        Box::new(FsThumbnailGenerator),
        Box::new(ImageCrateDecoder),
        Box::new(SystemClock),
        Box::new(BackgroundPreviewPipeline::new()),
    )
}

#[derive(Debug, Clone)]
enum Command {
    Ui,
    Import { folder: String },
    List,
    Open { image_id: i64 },
    ShowEdit { image_id: i64 },
    SetEdit { image_id: i64, params: EditParams },
}

#[derive(Debug, Clone)]
enum CommandError {
    Usage(String),
    Runtime(String),
}

fn parse_command(args: &[String]) -> Result<Command, CommandError> {
    if args.len() <= 1 {
        return Ok(Command::Ui);
    }

    match args[1].as_str() {
        "ui" => Ok(Command::Ui),
        "import" => {
            if args.len() < 3 {
                return Err(CommandError::Usage("missing folder path".to_string()));
            }
            Ok(Command::Import {
                folder: args[2].clone(),
            })
        }
        "list" => Ok(Command::List),
        "open" => {
            if args.len() < 3 {
                return Err(CommandError::Usage("missing image id".to_string()));
            }
            let image_id = args[2]
                .parse::<i64>()
                .map_err(|_| CommandError::Usage(format!("invalid image id: {}", args[2])))?;
            Ok(Command::Open { image_id })
        }
        "show-edit" => {
            if args.len() < 3 {
                return Err(CommandError::Usage("missing image id".to_string()));
            }
            let image_id = args[2]
                .parse::<i64>()
                .map_err(|_| CommandError::Usage(format!("invalid image id: {}", args[2])))?;
            Ok(Command::ShowEdit { image_id })
        }
        "set-edit" => {
            if args.len() != 9 {
                return Err(CommandError::Usage(
                    "set-edit requires 8 args: <image_id> <exposure> <contrast> <temperature> <tint> <highlights> <shadows>".to_string(),
                ));
            }
            let image_id = args[2]
                .parse::<i64>()
                .map_err(|_| CommandError::Usage(format!("invalid image id: {}", args[2])))?;
            let params = EditParams {
                exposure: parse_f32_arg("exposure", &args[3])?,
                contrast: parse_f32_arg("contrast", &args[4])?,
                temperature: parse_f32_arg("temperature", &args[5])?,
                tint: parse_f32_arg("tint", &args[6])?,
                highlights: parse_f32_arg("highlights", &args[7])?,
                shadows: parse_f32_arg("shadows", &args[8])?,
            };
            Ok(Command::SetEdit { image_id, params })
        }
        other => Err(CommandError::Usage(format!("unknown command: {other}"))),
    }
}

fn parse_f32_arg(name: &str, value: &str) -> Result<f32, CommandError> {
    value
        .parse::<f32>()
        .map_err(|_| CommandError::Usage(format!("invalid {name}: {value}")))
}

fn run_command(
    command: Result<Command, CommandError>,
    service: &ApplicationService,
    config: &AppConfig,
) -> Result<(), CommandError> {
    match command? {
        Command::Ui => {
            let images = service
                .list_images(ListImagesCommand)
                .map_err(|error| CommandError::Runtime(error.to_string()))?;
            let image_count = images.len();
            let active_image_id = images.first().map(|image| image.id);
            let active_image_path = images.first().map(|image| image.file_path.clone());
            let initial_params = if let Some(image_id) = active_image_id {
                service
                    .show_edit(ShowEditCommand { image_id })
                    .map_err(|error| CommandError::Runtime(error.to_string()))?
            } else {
                EditParams::default()
            };
            ui::launch_window(
                service,
                &config.catalog_path,
                &config.cache_dir,
                image_count,
                active_image_id,
                active_image_path,
                initial_params,
            )
            .map_err(CommandError::Runtime)
        }
        Command::Import { folder } => {
            let report = service
                .import_folder(ImportFolderCommand {
                    folder,
                    cache_root: config.cache_dir.clone(),
                })
                .map_err(|error| CommandError::Runtime(format!("import failed: {error}")))?;
            println!(
                "import finished: scanned={}, supported={}, newly_imported={}",
                report.scanned_files, report.supported_files, report.newly_imported
            );
            Ok(())
        }
        Command::List => {
            let images = service
                .list_images(ListImagesCommand)
                .map_err(|error| CommandError::Runtime(format!("list failed: {error}")))?;
            if images.is_empty() {
                println!("no images in catalog");
                return Ok(());
            }
            for image in images {
                println!("{}", present_image_row(&image));
            }
            Ok(())
        }
        Command::Open { image_id } => {
            let image_id = ImageId::new(image_id)
                .map_err(|error| CommandError::Usage(format!("invalid image id: {error}")))?;
            let decoded = service
                .open_image(OpenImageCommand { image_id })
                .map_err(|error| CommandError::Runtime(format!("open failed: {error}")))?;
            println!("{}", present_decoded(image_id.get(), &decoded));
            Ok(())
        }
        Command::ShowEdit { image_id } => {
            let image_id = ImageId::new(image_id)
                .map_err(|error| CommandError::Usage(format!("invalid image id: {error}")))?;
            let params = service
                .show_edit(ShowEditCommand { image_id })
                .map_err(|error| CommandError::Runtime(format!("show-edit failed: {error}")))?;
            println!("{}", present_edit_params(image_id.get(), &params));
            Ok(())
        }
        Command::SetEdit { image_id, params } => {
            let image_id = ImageId::new(image_id)
                .map_err(|error| CommandError::Usage(format!("invalid image id: {error}")))?;
            service
                .set_edit(SetEditCommand { image_id, params })
                .map_err(|error| CommandError::Runtime(format!("set-edit failed: {error}")))?;
            println!("{}", present_edit_params(image_id.get(), &params));
            Ok(())
        }
    }
}

fn print_usage() {
    println!("usage:");
    println!("  lite-room ui");
    println!("  lite-room import <folder>");
    println!("  lite-room list");
    println!("  lite-room open <image_id>");
    println!("  lite-room show-edit <image_id>");
    println!(
        "  lite-room set-edit <image_id> <exposure> <contrast> <temperature> <tint> <highlights> <shadows>"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_import_command() {
        let args = vec![
            "lite-room".to_string(),
            "import".to_string(),
            "photos".to_string(),
        ];
        let command = parse_command(&args).expect("import should parse");
        assert!(matches!(command, Command::Import { .. }));
    }

    #[test]
    fn parse_open_rejects_invalid_id() {
        let args = vec![
            "lite-room".to_string(),
            "open".to_string(),
            "abc".to_string(),
        ];
        let command = parse_command(&args);
        assert!(matches!(command, Err(CommandError::Usage(_))));
    }

    #[test]
    fn parse_set_edit_command() {
        let args = vec![
            "lite-room".to_string(),
            "set-edit".to_string(),
            "1".to_string(),
            "0.1".to_string(),
            "0.2".to_string(),
            "0.3".to_string(),
            "0.4".to_string(),
            "0.5".to_string(),
            "0.6".to_string(),
        ];
        let command = parse_command(&args).expect("set-edit should parse");
        assert!(matches!(command, Command::SetEdit { .. }));
    }
}
