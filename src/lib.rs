pub mod modules;
use anyhow::Result;
use std::io::{self, Write};
use config::{Config, File, FileFormat};
use std::collections::HashMap;

/// 
/// Read all data under the specified section from the designated file into a HashMap.
/// 
/// # Parameters
/// - `path`: Path to file.
/// 
/// # Returns
/// a HashMap with 
/// 
/// # Example
/// ```
/// let chassis_map = read_config_section(file, "chassis")?;
/// let board_map = read_config_section(file, "board")?;
/// let product_map = read_config_section(file, "product")?;
/// ```
pub
fn read_config_section(file: &str, section: &str) -> Result<HashMap<String, String>, config::ConfigError> {
    let builder = Config::builder()
        .add_source(File::new(file, FileFormat::Toml));
    let settings = builder.build()?;
    if let Some(section_values) = settings.get::<HashMap<String, String>>(section).ok() {
        Ok(section_values)
    } else {
        Err(config::ConfigError::NotFound(format!("Section '{}' not found", section)))
    }
}

pub fn load_yaml(file: &str) -> Result<HashMap<String, String>, config::ConfigError> {
    // Read the whole file as a HashMap with `Option<String>` values
    let settings = Config::builder()
        .add_source(File::new(file, FileFormat::Yaml))
        .build()?;

    // Deserialize into `HashMap<String, Option<String>>`
    let config_map: HashMap<String, Option<String>> = settings.try_deserialize()?;

    // Transform `Option<String>` values to `String`, defaulting to an empty string
    let result = config_map
        .into_iter()
        .map(|(k, v)| (k, v.unwrap_or_default()))
        .collect();

    Ok(result)
}

pub
fn write_encoded_data_to_bin_file(binary_data: &Vec<u8>, file: &str) -> io::Result<()>{
    let mut file = std::fs::File::create(file)?;
    file.write_all(&binary_data)?;
    Ok(())
}

