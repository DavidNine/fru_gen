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
use std::{fs::File, io::Write, path::PathBuf};
use clap::Parser;
use anyhow::Result;
use fru_gen::*;


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
    
    #[doc = r"Enable debug mode"]
    #[arg(short = 'd', long = "debug")]
    debug: bool,

    #[doc = r"Generate a default config template (default = 'fru_gen.toml')"]
    #[arg(short = 'b', long = "build-config")]
    build_config: Option<String>,
}

fn build_config_template(filename: &str) -> Result<()>{
    let mut file = File::create(filename)?;
    let default_content = r#"
[common]
file_size = 256

internal_area = "Disabled"
chassis_area = "Enabled"
board_area = "Enabled"
product_area = "Enabled"


[chassis]
type = "Rack Mount Chassis"
part_number = "CHS1234"
serial_number = "SN5678"

[board]
manufacturer = "qwfqwfg"
product_name = "Board124"
serial_number = "SN12345"
part_number = "BP9876"
fru_file_id = "FRU123"

[product]
manufacturer = "ProductMFC"
product_name = "Product1"
part_number = "PN5678"
version = "V1.0.0"
serial_number = "SN123456"
asset_tag = "AssetTag"
"#;

    file.write_all(default_content.as_bytes())?;
    Ok(())
}



fn main() -> Result<()> {
    

    // Argument parser
    let args = ToolArgument::parse();

    // Config_path
    let config_path_buf = args.path
            .unwrap_or_else(|| {
                if args.debug {
                    eprintln!("Warning: no specified config file. Using default config: 'fru_gen.toml'");
                }
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


    if !config_path_buf.exists() {
        eprintln!("Warning: default config file: {} could not be found, Creating default config file.", config_path);
        build_config_template("fru_gen.toml")
            .unwrap_or_else(|e| panic!("Error: Failed to builld config template, reason: '{}'", e));
        println!("Build config file 'fru_gen.toml' done.");
    }


    let fru_str = load_fru_data(config_path)
        .unwrap_or_else(|e| panic!("Load fru data from {} failed, reason:{}", config_path, e));
    
    
    let fru_data:Vec<u8> = process_fru_data(&fru_str, &config_path)
        .unwrap_or_else(|e| panic!("Error: Failed to process fru data, {e}"));

    // Write data
    write_encoded_data_to_bin_file(&fru_data, &args.file)
        .unwrap_or_else(|e| panic!("Error: Failed to write bin file, {e}"));
    
    println!("Generate fru file: '{}'", &args.file);
    println!("Done");
    Ok(())
}

