mod common;
use common::{FruData, Chassis, Board, Product};
use config::ConfigError;
use core::panic;
use std::{collections::HashMap, path::PathBuf};
use anyhow::{Context, Result};
use config::{Config, File, FileFormat};
use std::io::{self, Write};
use std::fmt::Write as FmtWrite;

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
fn string_to_hex(input: &str) -> String {
    let mut hex_string = String::new();
    for c in input.chars() {
        write!(&mut hex_string, "{:02X}", c as u8).unwrap();
    }
    hex_string
}

pub
fn show_fru_data(fru_data: &FruData) {
    println!("============================================================");
    println!(" * {:<30}: {}", "chassis type", fru_data.chassis.chassis_type);
    println!(" * {:<30}: {}", "chassis part number", fru_data.chassis.chassis_part_number);
    println!(" * {:<30}: {}", "chassis serial number", fru_data.chassis.chassis_serial_number);
    println!(" * {:<30}: {}", "board manufacturer", fru_data.board.board_manufacturer);
    println!(" * {:<30}: {}", "board product name", fru_data.board.board_product_name);
    println!(" * {:<30}: {}", "board serial number", fru_data.board.board_serial_number);
    println!(" * {:<30}: {}", "board part number", fru_data.board.board_part_number);
    println!(" * {:<30}: {}", "board fru file id", fru_data.board.board_fru_file_id);
    println!(" * {:<30}: {}", "product manufacturer", fru_data.product.product_manufacturer);
    println!(" * {:<30}: {}", "product product name", fru_data.product.product_product_name);
    println!(" * {:<30}: {}", "product part number", fru_data.product.product_part_number);
    println!(" * {:<30}: {}", "product version", fru_data.product.product_version);
    println!(" * {:<30}: {}", "product serial number", fru_data.product.product_serial_number);
    println!(" * {:<30}: {}", "product asset tag", fru_data.product.product_asset_tag);
    println!("============================================================");
}

pub 
fn fru_data_to_hex(fru_data: &FruData) -> String{
    let mut hex_result = String::new();
    
    hex_result.push_str(&string_to_hex(&fru_data.chassis.chassis_type));
    hex_result.push_str(&string_to_hex(&fru_data.chassis.chassis_part_number));
    hex_result.push_str(&string_to_hex(&fru_data.chassis.chassis_serial_number));
    hex_result.push_str(&string_to_hex(&fru_data.board.board_manufacturer));
    hex_result.push_str(&string_to_hex(&fru_data.board.board_product_name));
    hex_result.push_str(&string_to_hex(&fru_data.board.board_serial_number));
    hex_result.push_str(&string_to_hex(&fru_data.board.board_part_number));
    hex_result.push_str(&string_to_hex(&fru_data.board.board_fru_file_id));
    hex_result.push_str(&string_to_hex(&fru_data.product.product_manufacturer));
    hex_result.push_str(&string_to_hex(&fru_data.product.product_product_name));
    hex_result.push_str(&string_to_hex(&fru_data.product.product_part_number));
    hex_result.push_str(&string_to_hex(&fru_data.product.product_version));
    hex_result.push_str(&string_to_hex(&fru_data.product.product_serial_number));
    hex_result.push_str(&string_to_hex(&fru_data.product.product_asset_tag));

    hex_result
}

pub
fn transfer_chassis_type_str_to_code(chassis_type_str: &str) -> Option<u8>{
    let chassis_type_map = HashMap::from([
        ("Other"                , 0x01),
        ("Unknown"              , 0x02),
        ("Desktop"              , 0x03),
        ("Low Profile Desktop"  , 0x04),
        ("Pizza Box"            , 0x05),
        ("Mini Tower"           , 0x06),
        ("Tower"                , 0x07),
        ("Portable"             , 0x08),
        ("Laptop"               , 0x09),
        ("Notebook"             , 0x0A),
        ("Lunch Box"            , 0x10),
        ("Main Server Chassis"  , 0x11),
        ("Expansion Chassis"    , 0x12),
        ("SubChassis"           , 0x13),
        ("Bus Expansion Chassis", 0x14),
        ("Peripheral Chassis"   , 0x15),
        ("RAID Chassis"         , 0x16),
        ("Rack Mount Chassis"   , 0x17),
        ("Sealed-case PC"       , 0x18),
        ("Multi-system Chassis" , 0x19),
        ("Compact PCI"          , 0x1A),
        ("Advanced TCA"         , 0x1B),
        ("Blade"                , 0x1C),
        ("Blade Enclosure"      , 0x1D),
        ("Tablet"               , 0x1E),
        ("Convertible"          , 0x1F),
        ("Detachable"           , 0x20),
        ("IoT Gateway"          , 0x21),
        ("Embedded PC"          , 0x22),
        ("Mini PC"              , 0x23),
        ("Stick PC"             , 0x24),
    ]);

    chassis_type_map.get(chassis_type_str).copied()
}


pub
fn build_chassis_area(fru_data: &FruData) -> Vec<u8>{
    let mut chassis_area = Vec::new();

    // Chassis area header
    chassis_area.push(0x01);    // Format version
    chassis_area.push(0x00);    // Area length
    chassis_area.push(0x00);    // Chassis type (to be set)


    // Chassis type
    let chassis_type_code = transfer_chassis_type_str_to_code(&fru_data.chassis.chassis_type).unwrap_or(0x02);  // If string not found, default will be Unknow.
    chassis_area[2] = chassis_type_code;

    // Chassis Part Number
    chassis_area.push(0xC0 | fru_data.chassis.chassis_part_number.len() as u8);         // Chassis Part Number length
    if fru_data.chassis.chassis_part_number.len() > 0x3F {
        panic!("Error: String length of Chassis Part Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.chassis.chassis_part_number.len());
    }
    chassis_area.extend_from_slice(&fru_data.chassis.chassis_part_number.as_bytes());   // Chassis Part Number data

    
    // Chassis Serial Number
    chassis_area.push(0xC0 | fru_data.chassis.chassis_serial_number.len() as u8);       // Chassis Serial Number length
    if fru_data.chassis.chassis_serial_number.len() > 0x3F {
        panic!("Error: String length of Chassis Serial Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.chassis.chassis_serial_number.len());
    }
    chassis_area.extend_from_slice(&fru_data.chassis.chassis_serial_number.as_bytes()); // Chassis Serial Number data


    // End of Chassis area, 0xC1 as end Byte
    chassis_area.push(0xC1);


    // fill up the rest area space with 8 Byte
    while (chassis_area.len() % 8) != 0{
        chassis_area.push(0x00);
    }    

    // Update Area length
    chassis_area[1] = (chassis_area.len() / 8 ) as u8;      

    // Update checksum
    let checksum = (0x100u16 - (chassis_area.iter().map(|&b| b as u16).sum::<u16>() % 256)) % 256;  // Calculate checksum
    if let Some(last_byte) = chassis_area.last_mut() {
        *last_byte = checksum as u8;
    }

    chassis_area    // return chassis data. (Dtype = Vec<u8>)
}


pub
fn build_board_area(fru_data: &FruData) -> Vec<u8> {
    let mut board_area = Vec::new();

    // Board Area Header
    board_area.push(0x01);  // Format version
    board_area.push(0x00);  // Area lenght
    board_area.push(0x00);  // Language code ( 0 for English )
    board_area.push(0x00);  // Board Mfg. Date/Time ( 0 for unspecified )
    board_area.push(0x00);  // Board Mfg. Date/Time ( 0 for unspecified )
    board_area.push(0x00);  // Board Mfg. Date/Time ( 0 for unspecified )


    board_area.push(0xC0 | fru_data.board.board_manufacturer.len() as u8);
    if fru_data.board.board_manufacturer.len() > 0x3F {
        panic!("Error: String length of Board Manufacturer exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.board.board_manufacturer.len());
    }
    board_area.extend_from_slice(&fru_data.board.board_manufacturer.as_bytes());


    board_area.push(0xC0 | fru_data.board.board_product_name.len() as u8);
    if fru_data.board.board_product_name.len() > 0x3F {
        panic!("Error: String length of Board Product Name exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.board.board_product_name.len());
    }
    board_area.extend_from_slice(&fru_data.board.board_product_name.as_bytes());


    board_area.push(0xC0 | fru_data.board.board_serial_number.len() as u8);
    if fru_data.board.board_serial_number.len() > 0x3F {
        panic!("Error: String length of Board Serial Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.board.board_serial_number.len());
    }
    board_area.extend_from_slice(&fru_data.board.board_serial_number.as_bytes());


    board_area.push(0xC0 | fru_data.board.board_part_number.len() as u8);
    if fru_data.board.board_part_number.len() > 0x3F {
        panic!("Error: String length of Board Part Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.board.board_part_number.len());
    }
    board_area.extend_from_slice(&fru_data.board.board_part_number.as_bytes());


    board_area.push(0xC0 | fru_data.board.board_fru_file_id.len() as u8);
    if fru_data.board.board_fru_file_id.len() > 0x3F {
        panic!("Error: String length of Board Part Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.board.board_fru_file_id.len());
    }
    board_area.extend_from_slice(&fru_data.board.board_fru_file_id.as_bytes());


    board_area.push(0xC1);

    // fill up the rest area space with 8 Byte
    while (board_area.len() % 8) != 0{
        board_area.push(0x00);
    }    

    // Update Area length
    board_area[1] = (board_area.len() / 8 ) as u8;      

    // Update checksum
    let checksum = (0x100u16 - (board_area.iter().map(|&b| b as u16).sum::<u16>() % 256)) % 256;  // Calculate checksum
    if let Some(last_byte) = board_area.last_mut() {
        *last_byte = checksum as u8;
    }

    board_area
}


pub 
fn build_product_area(fru_data: &FruData) -> Vec<u8> {
    let mut product_area = Vec::new();

    // Product Area Header
    product_area.push(0x01);  // Format version
    product_area.push(0x00);  // Area lenght


    product_area.push(0xC0 | fru_data.product.product_manufacturer.len() as u8);
    if fru_data.product.product_manufacturer.len() > 0x3F {
        panic!("Error: String length of Product Manufacturer exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.product.product_manufacturer.len());
    }
    product_area.extend_from_slice(&fru_data.product.product_manufacturer.as_bytes());


    
    product_area.push(0xC0 | fru_data.product.product_product_name.len() as u8);
    if fru_data.product.product_product_name.len() > 0x3F {
        panic!("Error: String length of Product Name exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.product.product_product_name.len());
    }
    product_area.extend_from_slice(&fru_data.product.product_product_name.as_bytes());


    product_area.push(0xC0 | fru_data.product.product_part_number.len() as u8);
    if fru_data.product.product_part_number.len() > 0x3F {
        panic!("Error: String length of Product Part Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.product.product_part_number.len());
    }
    product_area.extend_from_slice(&fru_data.product.product_part_number.as_bytes());
    
    
    product_area.push(0xC0 | fru_data.product.product_version.len() as u8);
    if fru_data.product.product_version.len() > 0x3F {
        panic!("Error: String length of Product Version exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.product.product_version.len());
    }
    product_area.extend_from_slice(&fru_data.product.product_version.as_bytes());


    product_area.push(0xC0 | fru_data.product.product_serial_number.len() as u8);
    if fru_data.product.product_serial_number.len() > 0x3F {
        panic!("Error: String length of Product Serial Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.product.product_serial_number.len());
    }
    product_area.extend_from_slice(&fru_data.product.product_serial_number.as_bytes());
    
    
    product_area.push(0xC0 | fru_data.product.product_asset_tag.len() as u8);
    if fru_data.product.product_asset_tag.len() > 0x3F {
        panic!("Error: String length of Product Asset Tag exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.product.product_asset_tag.len());
    }
    product_area.extend_from_slice(&fru_data.product.product_asset_tag.as_bytes());

    
    product_area.push(0xC1);

    // fill up the rest area space with 8 Byte
    while (product_area.len() % 8) != 0{
        product_area.push(0x00);
    }    

    // Update Area length
    product_area[1] = (product_area.len() / 8 ) as u8;      

    // Update checksum
    let checksum = (0x100u16 - (product_area.iter().map(|&b| b as u16).sum::<u16>() % 256)) % 256;  // Calculate checksum
    if let Some(last_byte) = product_area.last_mut() {
        *last_byte = checksum as u8;
    }

    product_area
    
}




pub
fn write_encoded_data_to_bin_file(binary_data: &Vec<u8>, file: &str) -> io::Result<()>{
    let mut file = std::fs::File::create(file)?;
    file.write_all(&binary_data)?;
    Ok(())
}

