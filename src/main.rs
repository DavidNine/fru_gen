/**********************************************************************************\
    MIT License
    Copyright (c) 2024 Guanyan Wang

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
    THE SOFTWARE.
\***********************************************************************************/

use core::panic;
use std::io;
use std::{fs::File, io::Write, path::PathBuf};
use clap::Parser;
use anyhow::Result;
use fru_gen::*;
use modules::area::Area;
use modules::internal_area::Internal;
use modules::chassis_area::Chassis;
use modules::board_area::Board;
use modules::product_area::Product;


/*
    argument struct.
*/
#[derive(Parser, Debug)]
#[command(
author = "Guanyan.Wang", 
version = 
"utility v0.10 
Copyright (C) 2024 Guanyan Wang
    
A utility to generate FRU files compatible with IPMI tool usage.

For more information, please contact: ninebro1211@gmail.com
",
about = "A utility to generate FRU files compatible with IPMI tool usage."
)]

#[command(help_template = "
{about-with-newline}

USAGE:
    {usage}

OPTIONS:
{all-args}

EXAMPLE:
    * To generate a fru file by default.
        fru_gen
    
    * To generate a fru file called 'test.bin' with cofig named 'cs_fru.toml'.
        fru_gen -o test.bin -r cs_fru.toml
        fru_gen --output-file test.bin --read-config cs_fru.toml
")]
struct ToolArgument {

    #[doc = r"Specify output file name (default = 'fru_gen.bin')"]
    #[arg(short = 'o', long = "output-file", default_value = "fru_gen.bin")]
    file: String,

    #[doc = r"Specify config file path for generate custome FRU file"]
    #[arg(short = 'r', long = "read-config")]
    path: Option<std::path::PathBuf>,
    
    #[doc = r"Generate a default config template (default = 'fru_gen.toml')"]
    #[arg(short = 'b', long = "build-config")]
    build_config: Option<String>,

    #[doc = r"Enable debug mode"]
    #[arg(short = 'd', long = "debug")]
    debug: bool,
}

fn build_config_template(filename: &str) -> Result<()>{
    let mut file = File::create(filename)?;
    let default_content = r#"
[common]
file_size = 256

internal_area = "Enabled"
chassis_area = "Enabled"
board_area = "Enabled"
product_area = "Enabled"


[chassis]
type = "Rack Mount Chassis"
part_number = "CHS1234"
serial_number = "SN5678"
extra = "Chassis extra"

[board]
manufacturer = "qwfqwfg"
product_name = "Board124"
serial_number = "SN12345"
part_number = "BP9876"
fru_file_id = "FRU123"
extra = "Board extra"

[product]
manufacturer = "ProductMFC"
product_name = "Product1"
part_number = "PN5678"
version = "V1.0.0"
serial_number = "SN123456"
asset_tag = "AssetTag"
extra = "Product extra"

"#;

    file.write_all(default_content.as_bytes())?;
    Ok(())
}



pub
fn process_fru_data(config_path: &str, debug: bool) -> Result<Vec<u8>> {
    
    
    let mut fru_data = Vec::new();
    let common_area_setting_map = read_config_section(config_path, "common")?;

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
    

    let chassis_map = read_config_section(config_path, "chassis")?;
    let board_map = read_config_section(config_path, "board")?;
    let product_map = read_config_section(config_path, "product")?;


    let internal = Internal::new("".to_string());

    let chassis = Chassis::new(
        chassis_map.get("type").unwrap().to_string(),
        chassis_map.get("part_number").unwrap().to_string(),
        chassis_map.get("serial_number").unwrap().to_string(),
        chassis_map.get("extra").unwrap().to_string(),
    );
    
    let board = Board::new(
        board_map.get("manufacturer").unwrap().to_string(),
        board_map.get("product_name").unwrap().to_string(),
        board_map.get("serial_number").unwrap().to_string(),
        board_map.get("part_number").unwrap().to_string(),
        board_map.get("fru_file_id").unwrap().to_string(),
        board_map.get("extra").unwrap().to_string(),
    );

    let product = Product::new(
        product_map.get("manufacturer").unwrap().to_string(),
        product_map.get("product_name").unwrap().to_string(),
        product_map.get("part_number").unwrap().to_string(),
        product_map.get("version").unwrap().to_string(),
        product_map.get("serial_number").unwrap().to_string(),
        product_map.get("asset_tag").unwrap().to_string(),
        product_map.get("extra").unwrap().to_string(),
    );
    
    let internal_area_data = internal.transfer_as_byte();
    let chassis_area_data = chassis.transfer_as_byte();
    let board_area_data = board.transfer_as_byte();
    let product_area_data = product.transfer_as_byte();
    
    if debug == true {
        println!("{:?}", internal_area_data);
        println!("{:?}", chassis_area_data);
        println!("{:?}", board_area_data);
        println!("{:?}", product_area_data);
    }
    
    if common_area_setting_map.get("internal_area").unwrap().to_string() == "Enabled" {
        fru_data.extend(&internal_area_data);
        fru_data[1] = 0x01;
    }
    
    if common_area_setting_map.get("chassis_area").unwrap().to_string() == "Enabled" {
        fru_data.extend(&chassis_area_data);
        if fru_data[1] == 0 {
            fru_data[2] = 0x01;
        } else {
            if internal_area_data.len() % 8 == 0 {
                fru_data[2] = fru_data[1] as u8 + (internal_area_data.len() / 8) as u8;
            } else if internal_area_data.len() % 8 != 0 {
                fru_data[2] = fru_data[1] as u8 + (internal_area_data.len() / 8 + 1) as u8;
            }
        }
    }
    
    if common_area_setting_map.get("board_area").unwrap().to_string() == "Enabled" {
        fru_data.extend(&board_area_data);
        if fru_data[2] == 0 {
            fru_data[3] = 0x01;
        } else {
            if internal_area_data.len() % 8 == 0 {
                fru_data[3] = fru_data[2] as u8 + (chassis_area_data.len() / 8) as u8;      // Update board area start offset.
            } else if internal_area_data.len() % 8 != 0 {
                fru_data[3] = fru_data[2] as u8 + (chassis_area_data.len() / 8 + 1) as u8;  // Update board area start offset.
            }
        }
    }
    
    if common_area_setting_map.get("product_area").unwrap().to_string() == "Enabled" {
        fru_data.extend(&product_area_data);
        if fru_data[3] == 0 {
            fru_data[4] = 0x01;
        } else {
            if internal_area_data.len() % 8 == 0 {
                fru_data[4] = fru_data[3] as u8 + (board_area_data.len() / 8) as u8;    // Update Product area start offset.
            } else if internal_area_data.len() % 8 != 0 {
                fru_data[4] = fru_data[3] as u8 + (board_area_data.len() / 8 + 1) as u8;    // Update Product area start offset.
            }
        }
    }
    
    // Calculate common Header checksum
    fru_data[7] = ((0x100u16 - (fru_data.iter().take(7).map(|&b| b as u16).sum::<u16>() % 256)) % 256) as u8;
    

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

fn main() -> Result<()> {
    
    // Argument parser
    let args = ToolArgument::parse();

    // Config_path
    let config_path_buf = args.path
            .unwrap_or_else(|| {
                PathBuf::from("fru_gen.toml")
            });

    let config_path = config_path_buf
        .as_path()
        .to_str()
        .expect("Could not convert path to a valid UTF-8 string");

    
    // Build config process
    if let Some(config_filename) = &args.build_config {
        build_config_template(config_filename)
            .unwrap_or_else(|e| panic!("Error: Failed to builld config template, reason: '{}'", e));
        println!("Build config file '{}' done.", config_filename);
        return Ok(());
    }

    // Check file exist, if not exist, build a new one.
    if !config_path_buf.exists() {
        eprintln!("Warning: default config file: {} could not be found, Creating default config file.", config_path);
        build_config_template("fru_gen.toml")
            .unwrap_or_else(|e| panic!("Error: Failed to builld config template, reason: '{}'", e));
        println!("Build config file 'fru_gen.toml' done.");
    }

    let fru_data = process_fru_data(config_path, args.debug)
        .unwrap_or_else(|e| panic!("Error: Failed to process fru data: {}", e));

    // Write data
    write_encoded_data_to_bin_file(&fru_data, &args.file)
        .unwrap_or_else(|e| panic!("Error: Failed to write bin file, {e}"));
    
    println!("Generate fru file: '{}'", &args.file);
    println!("Done");
    Ok(())
}

