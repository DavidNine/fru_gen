use core::panic;
use std::path::PathBuf;
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



fn main() -> Result<()> {
    
    show_banner();
    
    let args = Cli::parse();
    let config_path_buf = args.path.unwrap_or_else(|| PathBuf::from("fruGen.toml"));
    let config_path = config_path_buf
        .as_path()
        .to_str()
        .expect("Could not convert path to a valid UTF-8 string");
    let fru_str = load_fru_data(config_path)
        .unwrap_or_else(|e| panic!("Load fru data from {} failed, reason:{}", config_path, e));
    if args.debug {
        show_fru_data(&fru_str);
    }


    let mut fru_data = Vec::new();
    
    //-----------------
    //  Common Header
    //-----------------
    fru_data.push(0x01);        // FRU format version
    fru_data.push(0x00);        // Internal area offset ( No use, set to 0 )
    fru_data.push(0x01);        // Chassis area offset
    fru_data.push(0x00);        // Board area offset
    fru_data.push(0x00);        // Product area offset
    fru_data.push(0x00);        // Multi Record area offset
    fru_data.push(0x00);        // Pad Byte area offset
    fru_data.push(0x00);        // Checksum
    
    let chassis_area_data = build_chassis_area(&fru_str);

    
    let board_area_data = build_board_area(&fru_str);

    
    let product_area_data = build_product_area(&fru_str);

    fru_data[3] = fru_data[2] as u8 + (chassis_area_data.len() / 8 + 1) as u8;  // Update board area start offset.
    fru_data[4] = fru_data[3] as u8 + (board_area_data.len() / 8 + 1) as u8;    // Update Product area start offset.

    
    // Calculate common Header checksum
    let common_header_checksum = (0x100u16 - (fru_data.iter().take(7).map(|&b| b as u16).sum::<u16>() % 256)) % 256;
    fru_data[7] = common_header_checksum as u8;

    
    fru_data.extend(chassis_area_data);
    fru_data.extend(board_area_data);
    fru_data.extend(product_area_data);
    
    // Calculate checksum before padding
    if fru_data.len() % 8 == 0 {
        for _i in 0..7 {
            fru_data.push(0x00);
        }
    } else {
        while (fru_data.len() % 8) != 0 {
            fru_data.push(0x00);
        }
    }
    let checksum_index = fru_data.len();
    println!("check sum index = {}", checksum_index);
    fru_data.push(0x00);
    
    let checksum = (0x100u16 - (fru_data.iter().map(|&b| b as u16).sum::<u16>() % 256)) % 256;
    println!("check sum  = {}", checksum);
    
    fru_data[checksum_index] = checksum as u8;

    // If needed, extend fru_data to 256 bytes
    while fru_data.len() < 256 {
        fru_data.push(0x00);
    }

    // Write data
    write_encoded_data_to_bin_file(&fru_data, &args.file)
        .unwrap_or_else(|e| panic!("Error: Failed to write bin file, {e}"));
    
    println!("Done");
    Ok(())
}

