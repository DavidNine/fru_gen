pub mod modules;
pub mod common;
use config::ConfigError;
use std::collections::HashMap;
use anyhow::Result;
use config::{Config, File, FileFormat};
use std::io::{self, Write};



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
fn load_fru_data(file: &str) -> Result<FruData, ConfigError>{
let chassis_map = read_config_section(file, "chassis")?;
let board_map = read_config_section(file, "board")?;
let product_map = read_config_section(file, "product")?;

/*
    Build chassis area struct
*/
let chassis = Chassis { 
    chassis_type: chassis_map.get("type").unwrap().to_string(),
    chassis_part_number: chassis_map.get("part_number").unwrap().to_string(),
    chassis_serial_number: chassis_map.get("serial_number").unwrap().to_string(),
};

/*
    Build board area struct
*/
let board = Board {
    board_manufacturer: board_map.get("manufacturer").unwrap().to_string(),
    board_product_name: board_map.get("product_name").unwrap().to_string(),
    board_serial_number: board_map.get("serial_number").unwrap().to_string(),
    board_part_number: board_map.get("part_number").unwrap().to_string(),
    board_fru_file_id: board_map.get("fru_file_id").unwrap().to_string(),
};


/*
    Build product area struct
*/
let product = Product {
    product_manufacturer: product_map.get("manufacturer").unwrap().to_string(),
    product_product_name: product_map.get("product_name").unwrap().to_string(),
    product_part_number: product_map.get("part_number").unwrap().to_string(),
    product_version: product_map.get("version").unwrap().to_string(),
    product_serial_number: product_map.get("serial_number").unwrap().to_string(),
    product_asset_tag: product_map.get("asset_tag").unwrap().to_string(),
};


/*
    Build Fru data struct
*/

Ok(FruData {
    chassis,
    board,
    product,
})


}


pub
fn process_fru_data(fru_str: &FruData, config_path: &str) -> Result<Vec<u8>> {
    
    
    let mut fru_data = Vec::new();
    let common_area_setting_map = read_config_section(config_path, "common")
        .unwrap_or_else(|e| panic!("Error: Failed to read common config section, reason: \'{}\'", e));

    let default_size = 1024;
    let fru_size = common_area_setting_map.get("file_size").and_then(|v| v.parse::<i32>().ok())
            .unwrap_or_else(|| {
                eprintln!("Warning: 'file_size' not found or invalid. Using default value: {}", default_size);
                default_size
            });

    // Common Header
    fru_data.push(0x01);        // FRU format version
    fru_data.push(0x00);        // Internal area offset ( No use, set to 0 )
    fru_data.push(0x00);        // Chassis area offset
    fru_data.push(0x00);        // Board area offset
    fru_data.push(0x00);        // Product area offset
    fru_data.push(0x00);        // Multi Record area offset
    fru_data.push(0x00);        // Pad Byte area offset
    fru_data.push(0x00);        // Checksum
    
    let mut chassis_area_data = Vec::new();
    let mut board_area_data = Vec::new();
    let mut product_area_data = Vec::new();
    
    if common_area_setting_map.get("chassis_area").unwrap().to_string() == "Enabled" {
        println!("Building chassis area data.");
        chassis_area_data.extend(Chassis::build(&fru_str));
        fru_data.extend(&chassis_area_data);
        if fru_data[1] == 0 {
            fru_data[2] = 0x01;
        }
    }
    
    if common_area_setting_map.get("board_area").unwrap().to_string() == "Enabled" {
        println!("Building board area data.");
        board_area_data.extend(modules::area_builder::build_board_area(&fru_str));
        fru_data.extend(&board_area_data);
        if fru_data[2] == 0 {
            fru_data[3] = 0x01;
        } else {
            fru_data[3] = fru_data[2] as u8 + (chassis_area_data.len() / 8 + 1) as u8;  // Update board area start offset.

        }
    }
    
    if common_area_setting_map.get("product_area").unwrap().to_string() == "Enabled" {
        println!("Building product area data.");
        product_area_data.extend(modules::area_builder::build_product_area(&fru_str));
        fru_data.extend(&product_area_data);

        if fru_data[3] == 0 {
            fru_data[4] = 0x01;
        } else {
            fru_data[4] = fru_data[3] as u8 + (board_area_data.len() / 8 + 1) as u8;    // Update Product area start offset.
        }
    }
    


    // Calculate common Header checksum
    let common_header_checksum = (0x100u16 - (fru_data.iter().take(7).map(|&b| b as u16).sum::<u16>() % 256)) % 256;
    fru_data[7] = common_header_checksum as u8;
    

    // Check fru_data size.
    if (fru_data.len() as i32) > fru_size {
        panic!("Error: fru data total size exceed limitation\nExp:[{}], Act:[{}]", fru_size, fru_data.len());
    }

    // If needed, extend size of fru_data to specified bytes
    while (fru_data.len() as i32) < fru_size {
        fru_data.push(0x00);
    }

    println!("Fru Size: {}", fru_size);

    Ok(fru_data)
}


pub
fn write_encoded_data_to_bin_file(binary_data: &Vec<u8>, file: &str) -> io::Result<()>{
    let mut file = std::fs::File::create(file)?;
    file.write_all(&binary_data)?;
    Ok(())
}

