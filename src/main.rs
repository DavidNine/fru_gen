use core::panic;
use std::{io::{self, Write}, path::PathBuf};
use clap::Parser;
use anyhow::Result;
use config::ConfigError;
use serde::Deserialize;
use std::fmt::Write as FmtWrite;
use fru_gen::read_config_section;


/*
    Define FRU data area struct
*/
#[derive(Debug, Deserialize)]
struct Chassis {
    chassis_type: String,
    chassis_part_number: String,
    chassis_serial_number:String,
}

#[derive(Debug, Deserialize)]
struct Board {
    board_manufacturer: String,
    board_product_name: String,
    board_serial_number: String,
    board_part_number: String,
    board_fru_file_id: String,
}

#[derive(Debug, Deserialize)]
struct Product {
    product_manufacturer: String,
    product_product_name: String,
    product_part_number: String,
    product_version: String,
    product_serial_number: String,
    product_asset_tag: String,   
}

#[derive(Debug, Deserialize)]
struct FruData {
    chassis: Chassis,
    board: Board,
    product: Product,
}




/*
    argument struct.
*/
#[derive(Parser, Debug)]
#[command(
    author = "Guanyan.Wang", 
    version = 
"v0.10
Copyright (C) 2024 Guanyan Wang
    
A utility to generate FRU files compatible with IPMI tool usage.

For more information, please contact: ninebro1211@gmail.com",

    about = "A utility to generate FRU files compatible with IPMI tool usage."
)]
struct Cli {

    file: String,

    #[arg(short = 'r', long = "read", help = "Specified config file path for generate custome FRU file")]
    path: Option<std::path::PathBuf>,
    
    #[arg(short = 'd', long = "debug", help = "Enable debug mode")]
    debug: bool,
    
}



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


fn string_to_hex(input: &str) -> String {
    let mut hex_string = String::new();
    for c in input.chars() {
        write!(&mut hex_string, "{:02X}", c as u8).unwrap();
    }
    hex_string
}


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


fn encode_fru_string(s: &str) -> Vec<u8> {
    let mut data = Vec::new();
    let len = s.len();

    if len == 0 {
        data.push(0x00);
        return data;
    }

    if len > 0x3F {
        panic!("String exceed limitation. (0x3F)");
    }

    let type_length = 0xC0 | (len as u8 &0x3F);
    data.push(type_length);
    data.extend_from_slice(s.as_bytes());

    data
}

fn write_encoded_data_to_bin_file(binary_data: &Vec<u8>, file: &str) -> io::Result<()>{
    let mut file = std::fs::File::create(file)?;
    file.write_all(&binary_data)?;
    Ok(())
}

fn main() -> Result<()> {
    
    let args = Cli::parse();


    let config_path_buf = args.path.unwrap_or_else(|| PathBuf::from("fruGen.toml"));
    let config_path = config_path_buf
        .as_path()
        .to_str()
        .expect("Could not convert path to a valid UTF-8 string");

    let fru_data = load_fru_data(config_path)
        .unwrap_or_else(|e| panic!("Load fru data from {} failed, reason:{}", config_path, e));


    let hex_data = fru_data_to_hex(&fru_data);

    if args.debug {
        show_fru_data(&fru_data);
        let encoded_data = encode_fru_string(&fru_data.chassis.chassis_serial_number);
        println!("{:#?}", &encoded_data);
        write_encoded_data_to_bin_file(&encoded_data, &args.file).unwrap();
        let mut count = 0;
        for i in (0..hex_data.len()).step_by(2) {
            if let Some(hex_pair) = hex_data.get(i..i+2) {
                print!("{} ", hex_pair);
                
                count += 1;
                
                if count % 16 == 0 {
                    println!("");
                } else if count % 8 == 0 {
                    print!("  ");
                }
            }
        }
    }
    
    println!("Done");
    Ok(())
}
