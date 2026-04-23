/**********************************************************************************\
    MIT License
    Copyright (c) 2026 Guanyan Wang

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.
\**********************************************************************************/

use clap::Parser;
use std::{io::{self, Write}, path::PathBuf};
use anyhow::Result;
use tempfile::NamedTempFile;

use fru_gen::modules::{
    board_area::Board, chassis_area::Chassis,
    fru_editor::{FRUEditor, UI, Line}, internal_area::Internal, product_area::Product,
    area::{Area, FieldConfig},
};

use fru_gen::{load_config, load_yaml, build_config_template, parse_chassis_type};


const VERSION: &str = "1.0.0";

const HELP_MESSAGE: &str = "\
{before-help}FRU_Gen {version}
{author}
{about}

USAGE:
    {usage}

{all-args}

TUI CONTROLS:
    Tab          Switch between [Editor] and [Settings] pages
    Ctrl+S       Save configuration to binary
    Esc          Exit the application
    ↑/↓          Navigate between fields
    ←/→          Move cursor within a field (Editor page)
    PageUp/Down  Scroll Hint/Hex View panels
    e            Toggle Enable/Disable field (Settings page)
    +/-          Adjust Reserved Bytes for field (Settings page)

EXAMPLES:
    1. Launch TUI to create/edit FRU data:
       fru_gen -u

    2. Load existing config and launch TUI:
       fru_gen -u -r test.toml

    3. Generate FRU binary from config with custom size:
       fru_gen -r test.yaml -o output.bin --size 4096

    4. Generate a default config template:
       fru_gen -b my_config.toml

{after-help}
";


#[derive(Parser, Debug)]
#[command(
    name          = "FRU_Gen",
    author        = "Guanyan Wang",
    version       = VERSION,
    help_template = HELP_MESSAGE,
)]
struct ToolArgument {

    #[doc = r"Specify output binary file name (default = 'fru_gen.bin')"]
    #[arg(short = 'o', long = "output-file", default_value = "fru_gen.bin")]
    file: String,

    #[doc = r"Specify config file path (automatically detects TOML/YAML format)"]
    #[arg(short = 'r', long = "read-config")]
    path: Option<std::path::PathBuf>,
    
    #[doc = r"Generate a default configuration template (e.g., 'fru_gen.toml')"]
    #[arg(short = 'b', long = "build-config")]
    build_config: Option<String>,

    #[doc = r"Enable verbose debug output"]
    #[arg(short = 'd', long = "debug")]
    debug: bool,

    #[doc = r"Launch the interactive TUI mode (supports live hex preview)"]
    #[arg(short = 'u', long = "ui")]
    user_interface_mode: bool,

    #[doc = r"Total size of the output FRU binary in bytes (default = 4096)"]
    #[arg(short = 's', long = "size", default_value = "4096")]
    size: usize
}

pub
fn process_fru_data(config_path: &str, size: usize, debug: bool, ui_settings: Option<&[Line]>) -> Result<Vec<u8>> {
    let mut fru_data = Vec::new();
    let fru_size = size;

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

    let chassis_type_string = config_map.get("chassis_type").map(|f| f.value()).unwrap_or_else(|| "0x02".to_string());
    let chassis_type_code = parse_chassis_type(&chassis_type_string);


    let chassis = Chassis::new(
        chassis_type_code,
        config_map.get("chassis_part_number").map(|f| f.value()).unwrap_or_default(),
        config_map.get("chassis_serial_number").map(|f| f.value()).unwrap_or_default(),
        config_map.get("chassis_extra").map(|f| f.value()).unwrap_or_default(),
    );

    let board = Board::new(
        config_map.get("board_mfg_date_time").map(|f| f.value()).unwrap_or_else(|| "0".to_string()),
        config_map.get("board_manufacturer").map(|f| f.value()).unwrap_or_default(),
        config_map.get("board_product_name").map(|f| f.value()).unwrap_or_default(),
        config_map.get("board_serial_number").map(|f| f.value()).unwrap_or_default(),
        config_map.get("board_part_number").map(|f| f.value()).unwrap_or_default(),
        config_map.get("board_fruid").map(|f| f.value()).unwrap_or_default(),
        config_map.get("board_extra").map(|f| f.value()).unwrap_or_default(),
    );
    
    let product = Product::new(
        config_map.get("product_manufacturer").map(|f| f.value()).unwrap_or_default(),
        config_map.get("product_name").map(|f| f.value()).unwrap_or_default(),
        config_map.get("product_part_number").map(|f| f.value()).unwrap_or_default(),
        config_map.get("product_version").map(|f| f.value()).unwrap_or_default(),
        config_map.get("product_serial_number").map(|f| f.value()).unwrap_or_default(),
        config_map.get("product_asset_tag").map(|f| f.value()).unwrap_or_default(),
        config_map.get("product_fruid").map(|f| f.value()).unwrap_or_default(),
        config_map.get("product_extra").map(|f| f.value()).unwrap_or_default(),
    );
    
    let get_configs = |range: std::ops::Range<usize>| -> Vec<FieldConfig> {
        if let Some(lines) = ui_settings {
            lines[range].iter().map(|l: &Line| FieldConfig {
                enabled: l.enabled(),
                reserved_bytes: l.reserved_bytes(),
            }).collect()
        } else {
            let fields = vec![
                "chassis_type", "chassis_part_number", "chassis_serial_number", "chassis_extra",
                "board_mfg_date_time", "board_manufacturer", "board_product_name", "board_serial_number", "board_part_number", "board_fruid", "board_extra",
                "product_manufacturer", "product_name", "product_part_number", "product_version", "product_serial_number", "product_asset_tag", "product_fruid", "product_extra"
            ];
            
            range.map(|i| {
                let key = fields[i];
                let is_code = key.contains("type") || key.contains("mfg");
                let default_reserve = if is_code { 0 } else { 32 };
                
                if let Some(field) = config_map.get(key) {
                    FieldConfig {
                        enabled: true,
                        reserved_bytes: field.reserve_bytes().unwrap_or(default_reserve),
                    }
                } else {
                    FieldConfig {
                        enabled: false,
                        reserved_bytes: default_reserve,
                    }
                }
            }).collect()
        }
    };
    
    let internal_area_data  = internal.transfer_as_byte();
    let chassis_area_data   = chassis.transfer_with_config(&get_configs(0..4));
    let board_area_data     = board.transfer_with_config(&get_configs(4..11));
    let product_area_data   = product.transfer_with_config(&get_configs(11..19));

    if debug == true {
        println!("{:#?}", config_map);
        println!("{:?}", internal_area_data);
        println!("{:?}", chassis_area_data);
        println!("{:?}", board_area_data);
        println!("{:?}", product_area_data);
    }

    let mut current_offset = 1u8; // Start after Common Header (8 bytes)

    // Internal Area
    if !internal_area_data.is_empty() {
        fru_data[1] = current_offset;
        fru_data.extend(&internal_area_data);
        current_offset += (internal_area_data.len() / 8) as u8;
    }

    // Chassis Area
    if !chassis_area_data.is_empty() {
        fru_data[2] = current_offset;
        fru_data.extend(&chassis_area_data);
        current_offset += (chassis_area_data.len() / 8) as u8;
    }

    // Board Area
    if !board_area_data.is_empty() {
        fru_data[3] = current_offset;
        fru_data.extend(&board_area_data);
        current_offset += (board_area_data.len() / 8) as u8;
    }

    // Product Area
    if !product_area_data.is_empty() {
        fru_data[4] = current_offset;
        fru_data.extend(&product_area_data);
    }

    
    // Calculate common Header checksum
    fru_data[7] = ((0x100u16 - (fru_data.iter().take(7).map(|&b| b as u16).sum::<u16>() % 256)) % 256) as u8;
    

    // Check fru_data size.
    if fru_data.len() > fru_size {
        panic!("Error: fru data total size exceed limitation\nExp:[{}], Act:[{}]", fru_size, fru_data.len());
    }

    // If needed, extend size of fru_data to specified bytes
    while fru_data.len() < fru_size {
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


fn dispatch_function(args: &ToolArgument) -> Result<()> {
    if args.user_interface_mode {
        let initial_data = if let Some(path) = &args.path {
            let config_path = path.to_str().unwrap_or("output.yaml");
            if path.exists() {
                Some(load_config(config_path)?)
            } else {
                println!("Warning: Provided config file '{}' does not exist. Starting with empty fields.", config_path);
                None
            }
        } else {
            None
        };
        
        let fru_editor: FRUEditor = FRUEditor::new("FRU Editor".to_string());
        let temp_file = NamedTempFile::new()?; // Keeps the temporary file alive
        let temp_file_name = temp_file.path().to_str().unwrap_or("temp.yaml"); // Get the path as a string
        
        let settings = fru_editor.run(temp_file_name, initial_data)?;
        
        if let Some(s) = settings {
            let fru_data: Vec<u8> = process_fru_data(&temp_file_name, args.size, args.debug, Some(&s))?;
            write_encoded_data_to_bin_file(&fru_data, &args.file)?;
            println!("Generate fru file: '{}'", &args.file);
        } else {
            println!("No changes saved. Exiting.");
            return Ok(());
        }
    } else if let Some(config_filename) = &args.build_config {
        build_config_template(config_filename)?;
        println!("Build config file '{}' done.", config_filename);
        return Ok(());
    } else {
        // Config_path
        let config_path_buf = args.path.clone().unwrap_or_else(|| PathBuf::from("output.yaml"));
        
        if !config_path_buf.exists() {
            return Err(anyhow::anyhow!("Configuration file '{}' not found.
Hint: Use '-b' to generate a default template or '-u' to use the editor interface.", config_path_buf.display()));
        }

        let config_path = config_path_buf.as_path().to_str().unwrap_or_else(|| panic!("Could not convert path to a valid UTF-8 string"));
        let fru_data: Vec<u8> = process_fru_data(config_path, args.size, args.debug, None)?;

        // Write data
        write_encoded_data_to_bin_file(&fru_data, &args.file)?;
        println!("Generate fru file: '{}'", &args.file);
    }

    println!("Done");
    Ok(())
}


fn main() -> Result<()> {
    
    // Argument parser
    let args: ToolArgument = ToolArgument::parse();
    if let Err(e) = dispatch_function(&args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    Ok(())
}
