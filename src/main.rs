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

use std::io;
use anyhow::Result;
use clap::Parser;
use fru_gen::*;
use modules::{area::Area, fru_editor::UI};
use modules::internal_area::Internal;
use modules::chassis_area::Chassis;
use modules::board_area::Board;
use modules::product_area::Product;
use modules::fru_editor::FRUEditor;
use tempfile::NamedTempFile;

use std::{
    io::Write, 
    path::PathBuf
};



#[derive(Parser, Debug)]
#[command(
    author  = "Guanyan.Wang", 
    version = 

"utility v0.12
Copyright (C) 2024 Guanyan Wang
    
A utility to generate FRU files compatible with IPMI tool usage.

For more information, please contact: ninebro1211@gmail.com
",

    about = "A utility to generate FRU files compatible with IPMI tool usage."
)]

#[command(help_template = "
{about-with-newline}

Usage:
    {usage}

{all-args}

Example:
    * To generate a FRU file by default.
        $ fru_gen
    
    * To generate a FRU file called 'test.bin' with cofig named 'cs_fru.toml'.
        $ fru_gen -o test.bin -r cs_fru.toml
        $ fru_gen --output-file test.bin --read-config cs_fru.toml

    * To generate a FRU file with editor.
        $ fru_gen --ui
        $ fru_gen --ui -o test.bin
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

    #[doc = r"Generate FRU binary file by using TUI mode"]
    #[arg(short = 'u', long = "ui")]
    user_interface_mode: bool
}

pub
fn process_fru_data(config_path: &str, debug: bool) -> Result<Vec<u8>> {
    let mut fru_data = Vec::new();
    let fru_size = 256;

    // Common Header
    fru_data.push(0x01);        // FRU format version
    fru_data.push(0x00);        // Internal area offset ( No use, set to 0 )
    fru_data.push(0x00);        // Chassis area offset
    fru_data.push(0x00);        // Board area offset
    fru_data.push(0x00);        // Product area offset
    fru_data.push(0x00);        // Multi Record area offset
    fru_data.push(0x00);        // Pad Byte area offset
    fru_data.push(0x00);        // Checksum
    

    let config_map = load_yaml(config_path)?;
    let internal = Internal::new("".to_string());

    let chassis = Chassis::new(
        config_map.get("chassis_type").unwrap_or(&"".to_string()).to_string(),
        config_map.get("chassis_part_number").unwrap_or(&"".to_string()).to_string(),
        config_map.get("chassis_serial_number").unwrap_or(&"".to_string()).to_string(),
        config_map.get("chassis_extra").unwrap_or(&"".to_string()).to_string(),
    );

    let board = Board::new(
        config_map.get("board_manufacturer").unwrap_or(&"".to_string()).to_string(),
        config_map.get("board_product_Name").unwrap_or(&"".to_string()).to_string(),
        config_map.get("board_serial_Number").unwrap_or(&"".to_string()).to_string(),
        config_map.get("board_part_Number").unwrap_or(&"".to_string()).to_string(),
        config_map.get("board_fruid").unwrap_or(&"".to_string()).to_string(),
        config_map.get("board_extra").unwrap_or(&"".to_string()).to_string(),
    );
    
    let product = Product::new(
        config_map.get("product_manufacturer").unwrap_or(&"".to_string()).to_string(),
        config_map.get("product_name").unwrap_or(&"".to_string()).to_string(),
        config_map.get("product_part_number").unwrap_or(&"".to_string()).to_string(),
        config_map.get("product_version").unwrap_or(&"".to_string()).to_string(),
        config_map.get("product_serial_number").unwrap_or(&"".to_string()).to_string(),
        config_map.get("product_asset_tag").unwrap_or(&"".to_string()).to_string(),
        config_map.get("product_extra").unwrap_or(&"".to_string()).to_string(),
    );
    
    
    let internal_area_data  = internal.transfer_as_byte();
    let chassis_area_data   = chassis.transfer_as_byte();
    let board_area_data     = board.transfer_as_byte();
    let product_area_data   = product.transfer_as_byte();
    
    if debug == true {
        println!("{:#?}", config_map);
        println!("{:?}", internal_area_data);
        println!("{:?}", chassis_area_data);
        println!("{:?}", board_area_data);
        println!("{:?}", product_area_data);
    }
    
    fru_data.extend(&internal_area_data);
    fru_data[1] = 0x01;

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


///
fn write_encoded_data_to_bin_file(binary_data: &Vec<u8>, file: &str) -> io::Result<()>{
    let mut file = std::fs::File::create(file)?;
    file.write_all(&binary_data)?;
    Ok(())
}


fn dispatch_function(args: &ToolArgument) -> Result<(), io::Error> {
    if args.user_interface_mode {
        
        let fru_editor: FRUEditor = FRUEditor::new("FRU Editor".to_string());
        let temp_file = NamedTempFile::new()?; // Keeps the temporary file alive
        let temp_file_name = temp_file.path().to_str().unwrap_or("temp.yaml"); // Get the path as a string
        
        fru_editor.run(temp_file_name)?;
        let fru_data: Vec<u8> = process_fru_data(&temp_file_name, args.debug)
            .unwrap_or_else(|e| panic!("Error: Failed to process fru data: {}", e));

        // Write data
        write_encoded_data_to_bin_file(&fru_data, &args.file)
            .unwrap_or_else(|e| panic!("Error: Failed to write bin file, {e}"));

    } else if let Some(config_filename) = &args.build_config {
        build_config_template(config_filename).unwrap_or_else(|e| panic!("Error: Failed to builld config template, reason: '{}'", e));
        println!("Build config file '{}' done.", config_filename);
        return Ok(());
    } else {
        // Config_path
        let config_path_buf = args.path.clone().unwrap_or_else(|| PathBuf::from("output.yaml"));
        let config_path = config_path_buf.as_path().to_str().unwrap_or_else(|| panic!("Could not convert path to a valid UTF-8 string"));
        let fru_data: Vec<u8> = process_fru_data(config_path, args.debug).unwrap_or_else(|e| panic!("Error: Failed to process fru data: {}", e));

        // Write data
        write_encoded_data_to_bin_file(&fru_data, &args.file).unwrap_or_else(|e| panic!("Error: Failed to write bin file, {}", e));
    }

    println!("Generate fru file: '{}'", &args.file);
    println!("Done");
    Ok(())
}


fn main() -> Result<(), io::Error> {
    
    // Argument parser
    let args: ToolArgument = ToolArgument::parse();
    dispatch_function(&args)?;
    Ok(())
}

