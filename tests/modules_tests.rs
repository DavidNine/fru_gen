use fru_gen::modules::area::{Area, FieldConfig};
use fru_gen::modules::chassis_area::Chassis;
use fru_gen::modules::board_area::{Board, parse_mfg_time};
use fru_gen::modules::product_area::Product;

#[test]
fn test_chassis_area_generation() {
    let chassis = Chassis::new(
        0x11,
        "PART123".to_string(),
        "SN123".to_string(),
        "EXTRA".to_string(),
    );
    
    let configs = vec![
        FieldConfig { enabled: true, reserved_bytes: 0 },
        FieldConfig { enabled: true, reserved_bytes: 8 },
        FieldConfig { enabled: true, reserved_bytes: 8 },
        FieldConfig { enabled: true, reserved_bytes: 8 },
    ];
    
    let bytes = chassis.transfer_with_config(&configs);
    
    assert_eq!(bytes[0], 0x01); // Version
    assert_eq!(bytes[2], 0x11); // Type
    assert_eq!(bytes.len() % 8, 0); // Multiple of 8
    
    // Check for "PART123" with padding (total 8 bytes + 1 length byte)
    // 0xC0 | 8 = 0xC8
    let found_part = bytes.windows(8).any(|w| w == b"PART123 ");
    assert!(found_part);
}

#[test]
fn test_board_area_mfg_time() {
    // 1996-01-01 00:00:00 is 0
    assert_eq!(parse_mfg_time("19960101000000"), 0);
    // One minute later
    assert_eq!(parse_mfg_time("19960101000100"), 1);
    // Raw hex
    assert_eq!(parse_mfg_time("0x10"), 16);
    // Raw decimal
    assert_eq!(parse_mfg_time("100"), 100);
}

#[test]
fn test_board_area_generation() {
    let board = Board::new(
        "0".to_string(),
        "MFG".to_string(),
        "PROD".to_string(),
        "SN".to_string(),
        "PN".to_string(),
        "FRUID".to_string(),
        "EXTRA".to_string(),
    );
    
    let configs = vec![
        FieldConfig { enabled: true, reserved_bytes: 0 }, // mfg time
        FieldConfig { enabled: true, reserved_bytes: 4 },
        FieldConfig { enabled: true, reserved_bytes: 4 },
        FieldConfig { enabled: true, reserved_bytes: 4 },
        FieldConfig { enabled: true, reserved_bytes: 4 },
        FieldConfig { enabled: true, reserved_bytes: 4 },
        FieldConfig { enabled: true, reserved_bytes: 4 },
    ];
    
    let bytes = board.transfer_with_config(&configs);
    assert_eq!(bytes[0], 0x01);
    assert_eq!(bytes.len() % 8, 0);
}

#[test]
fn test_product_area_generation() {
    let product = Product::new(
        "MFG".to_string(),
        "NAME".to_string(),
        "PN".to_string(),
        "VER".to_string(),
        "SN".to_string(),
        "TAG".to_string(),
        "FRUID".to_string(),
        "EXTRA".to_string(),
    );
    
    let configs = vec![
        FieldConfig { enabled: true, reserved_bytes: 4 },
        FieldConfig { enabled: true, reserved_bytes: 4 },
        FieldConfig { enabled: true, reserved_bytes: 4 },
        FieldConfig { enabled: true, reserved_bytes: 4 },
        FieldConfig { enabled: true, reserved_bytes: 4 },
        FieldConfig { enabled: true, reserved_bytes: 4 },
        FieldConfig { enabled: true, reserved_bytes: 4 },
        FieldConfig { enabled: true, reserved_bytes: 4 },
    ];
    
    let bytes = product.transfer_with_config(&configs);
    assert_eq!(bytes[0], 0x01);
    assert_eq!(bytes.len() % 8, 0);
}
