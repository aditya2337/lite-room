pub fn launch_window(
    catalog_path: &str,
    cache_dir: &str,
    image_count: usize,
) -> Result<(), String> {
    println!("lite-room ui is not enabled in this build");
    println!("catalog: {catalog_path}");
    println!("cache: {cache_dir}");
    println!("images in catalog: {image_count}");
    Ok(())
}
