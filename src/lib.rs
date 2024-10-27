use std::{collections::HashMap, path::PathBuf};
use anyhow::{Context, Result};
use config::{Config, File, FileFormat};

pub 
fn read_config_file(path: Option<PathBuf>) -> Result<String> {
    let config_content = if let Some(path) = path {

        /*======================================
                    Path is provided
        ========================================*/
        
        if !path.exists() {
            anyhow::bail!("Specified path '{}' does not exist", path.display());
        } else if !path.is_file() {
            anyhow::bail!("Sepcified path '{}' is not regular file", path.display());
        }
        
        std::fs::read_to_string(&path)
        .with_context(|| format!("Could not read file '{}'", path.display()))?
        
    } else {

        /*======================================
                    Path is NOT provided
        ========================================*/

        let default_path = std::path::PathBuf::from("fruGen.CFG");
        
        if !default_path.exists() {
            anyhow::bail!("Default path '{}' does not exist", &default_path.display());
        } else if !default_path.is_file() {
            anyhow::bail!("Default path '{}' is not a regular file", &default_path.display());
        }

        std::fs::read_to_string("fruGen.CFG")
        .with_context(|| format!("Could not read default file '{}'", &default_path.display()))?
    };
    Ok(config_content)
}



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


pub
fn build_chassis_area() {
    
}