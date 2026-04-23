pub mod modules;
use anyhow::Result;
use config::{Config, File, FileFormat};
use std::collections::HashMap;
use std::io::Write;

///
/// Read all data under the specified section from the designated file into a HashMap.
///
/// # Parameters
/// - `file`: Name of configure file.
///
/// # Returns
/// a HashMap with
///
/// # Example
/// ```no_run
/// use fru_gen::read_config_section;
///
/// let file = "fru_gen.toml";
/// let chassis_map = read_config_section(file, "chassis")?;
/// let board_map = read_config_section(file, "board")?;
/// let product_map = read_config_section(file, "product")?;
/// # Ok::<(), config::ConfigError>(())
/// ```
pub fn read_config_section(
    file: &str,
    section: &str,
) -> Result<HashMap<String, String>, config::ConfigError> {
    let builder = Config::builder().add_source(File::new(file, FileFormat::Toml));
    let settings = builder.build()?;
    if let Some(section_values) = settings.get::<HashMap<String, String>>(section).ok() {
        Ok(section_values)
    } else {
        Err(config::ConfigError::NotFound(format!(
            "Section '{}' not found",
            section
        )))
    }
}

///
/// Transfer yaml file into a HashMap.
///
/// # Parameters
/// - `file`: Name of configure file.
///
/// # Returns
/// `HashMap`
///
/// # Example
/// ```no_run
/// use fru_gen::load_yaml;
///
/// let file = "output.yaml";
/// let config_map = load_yaml(file)?;
/// # Ok::<(), config::ConfigError>(())
/// ```
pub fn load_config(file: &str) -> Result<HashMap<String, String>, config::ConfigError> {
    let path = std::path::Path::new(file);
    let format = match path.extension().and_then(|s| s.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("toml") => FileFormat::Toml,
        Some(ext) if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") => FileFormat::Yaml,
        _ => FileFormat::Yaml,
    };

    let builder = Config::builder().add_source(File::new(file, format));
    let settings = builder.build()?;

    // Try to deserialize directly into HashMap<String, String> first
    if let Ok(config_map) = settings.clone().try_deserialize::<HashMap<String, String>>() {
        return Ok(config_map
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect());
    }

    // Fallback for cases where values might be null/missing in YAML or complex types in TOML
    // We try to get all values as a Map of Values and then convert them to strings
    let config_map: HashMap<String, config::Value> = settings.try_deserialize()?;
    Ok(config_map
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), v.to_string()))
        .collect())
}

pub fn load_yaml(file: &str) -> Result<HashMap<String, String>, config::ConfigError> {
    load_config(file)
}

pub fn build_config_template(filename: &str) -> Result<()> {
    let mut file = std::fs::File::create(filename)?;
    let default_content = r#"
Chassis_type = "Rack Mount Chassis"
Chassis_Part_Number = "CHS1234"
Chassis_Serial_Number = "SN5678"
Chassis_Extra = "Chassis extra"
Board_mfg_date_time = "0"
Board_Manufacturer = "qwfqwfg"
Board_Product_Name = "Board124"
Board_Serial_Number = "SN12345"
Board_Part_Number = "BP9876"
Board_Fruid = "FRU123"
Board_Extra = "Board extra"
Product_Manufacturer = "ProductMFC"
Product_Name = "Product1"
Product_Part_Number = "PN5678"
Product_Version = "V1.0.0"
Product_Serial_Number = "SN123456"
Product_Asset_Tag = "AssetTag"
Product_Fruid = "PFRU123"
Product_Extra = "Product extra"
"#;

    file.write_all(default_content.as_bytes())?;
    Ok(())
}

pub fn parser_hex_string(hex_str: &str) -> Result<u8, std::num::ParseIntError> {
    let trimmed = hex_str.trim();
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        u8::from_str_radix(&trimmed[2..], 16)
    } else {
        u8::from_str_radix(trimmed, 16)
    }
}

pub const CHASSIS_TYPE_TABLE: &[&str] = &[
    "Other",
    "Unknown",
    "Desktop",
    "Low Profile Desktop",
    "Pizza Box",
    "Mini Tower",
    "Tower",
    "Portable",
    "Laptop",
    "Notebook",
    "Lunch Box",
    "Main Server Chassis",
    "Expansion Chassis",
    "SubChassis",
    "Bus Expansion Chassis",
    "Peripheral Chassis",
    "RAID Chassis",
    "Rack Mount Chassis",
    "Sealed-case PC",
    "Multi-system Chassis",
    "Compact PCI",
    "Advanced TCA",
    "Blade",
    "Blade Enclosure",
    "Tablet",
    "Convertible",
    "Detachable",
    "IoT Gateway",
    "Embedded PC",
    "Mini PC",
    "Stick PC",
];

pub fn parse_chassis_type(input: &str) -> u8 {
    // Try parsing as hex first
    if let Ok(code) = parser_hex_string(input) {
        return code;
    }

    // Try matching against the table
    for (i, &name) in CHASSIS_TYPE_TABLE.iter().enumerate() {
        if name.eq_ignore_ascii_case(input.trim()) {
            return i as u8;
        }
    }

    // Default to 0x02 (Unknown) if all fails
    0x02
}
