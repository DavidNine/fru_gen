use fru_gen::{load_config, parse_chassis_type, parser_hex_string};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_parser_hex_string() {
    assert_eq!(parser_hex_string("0x12").unwrap(), 0x12);
    assert_eq!(parser_hex_string("12").unwrap(), 0x12);
    assert_eq!(parser_hex_string(" 0XFF ").unwrap(), 0xFF);
    assert!(parser_hex_string("G").is_err());
}

#[test]
fn test_parse_chassis_type() {
    // Test hex string input
    assert_eq!(parse_chassis_type("0x11"), 17); // Rack Mount Chassis
    
    // Test known names
    assert_eq!(parse_chassis_type("Rack Mount Chassis"), 17);
    assert_eq!(parse_chassis_type("  blade  "), 22);
    
    // Test unknown names (defaults to 0x02)
    assert_eq!(parse_chassis_type("Super Computer"), 2);
}

#[test]
fn test_load_config_toml() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.toml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "key = \"value\"").unwrap();
    writeln!(file, "Board_Manufacturer = \"MyMfg\"").unwrap();

    let config = load_config(file_path.to_str().unwrap()).unwrap();
    assert_eq!(config.get("key").unwrap().value(), "value");
    assert_eq!(config.get("board_manufacturer").unwrap().value(), "MyMfg");
}

#[test]
fn test_load_config_yaml() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "key: value").unwrap();
    writeln!(file, "Chassis_Part_Number: \"CPN123\"").unwrap();

    let config = load_config(file_path.to_str().unwrap()).unwrap();
    assert_eq!(config.get("key").unwrap().value(), "value");
    assert_eq!(config.get("chassis_part_number").unwrap().value(), "CPN123");
}
