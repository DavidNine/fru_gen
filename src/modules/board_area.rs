use chrono::{TimeZone, Utc};
use super::area::{Area, FieldConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Board {
    pub board_mfg_date_time: String,
    pub board_manufacturer: String,
    pub board_product_name: String,
    pub board_serial_number: String,
    pub board_part_number: String,
    pub board_fru_file_id: String,
    pub board_extra: String,
}

pub fn parse_mfg_time(input: &str) -> u32 {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return 0;
    }

    // Try parsing as YYYYMMDDHHMMSS (14 digits)
    if trimmed.len() == 14 && trimmed.chars().all(|c| c.is_ascii_digit()) {
        let year = trimmed[0..4].parse::<i32>().unwrap_or(1996);
        let month = trimmed[4..6].parse::<u32>().unwrap_or(1);
        let day = trimmed[6..8].parse::<u32>().unwrap_or(1);
        let hour = trimmed[8..10].parse::<u32>().unwrap_or(0);
        let min = trimmed[10..12].parse::<u32>().unwrap_or(0);
        let sec = trimmed[12..14].parse::<u32>().unwrap_or(0);

        if let Some(dt) = Utc.with_ymd_and_hms(year, month, day, hour, min, sec).single() {
            let epoch = Utc.with_ymd_and_hms(1996, 1, 1, 0, 0, 0).single().unwrap();
            if dt >= epoch {
                let diff = dt - epoch;
                return diff.num_minutes() as u32;
            }
        }
    }

    // Fallback to raw hex or decimal
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        u32::from_str_radix(&trimmed[2..], 16).unwrap_or(0)
    } else {
        trimmed.parse::<u32>().unwrap_or(0)
    }
}

impl Board {
    pub fn new(
        board_mfg_date_time: String,
        board_manufacturer: String,
        board_product_name: String,
        board_serial_number: String,
        board_part_number: String,
        board_fru_file_id: String,
        board_extra: String,
    ) -> Self {
        Board {
            board_mfg_date_time,
            board_manufacturer,
            board_product_name,
            board_serial_number,
            board_part_number,
            board_fru_file_id,
            board_extra,
        }
    }

    pub fn print_all(&self) {
        println!("Board Mfg Date Time = {}", &self.board_mfg_date_time);
        println!("Board Manufacturer  = {}", &self.board_manufacturer);
        println!("Board Product Name  = {}", &self.board_product_name);
        println!("Board Serial Number = {}", &self.board_serial_number);
        println!("Board Part Number   = {}", &self.board_part_number);
        println!("Board Fru ID        = {}", &self.board_fru_file_id);
        println!("Board Extra         = {}", &self.board_extra);
    }
}

impl Area for Board {
    fn check_area_length(&self, field_name: &str, field_value: &str) {
        if field_value.len() > 0x3F {
            panic!(
                "Error: String length of {} exceed limitation\nExp:[0x3F], Act:[0x{:02X}]",
                field_name,
                field_value.len()
            );
        }
    }

    fn validate(&self) {
        self.check_area_length("Board Manufacturer", &self.board_manufacturer);
        self.check_area_length("Board Product Name", &self.board_product_name);
        self.check_area_length("Board Serial Number", &self.board_serial_number);
        self.check_area_length("Board Part Number", &self.board_part_number);
        self.check_area_length("Board Fru ID", &self.board_fru_file_id);
        self.check_area_length("Board Extra", &self.board_extra);
    }

    fn transfer_as_byte(&self) -> Vec<u8> {
        let defaults = vec![
            FieldConfig { enabled: true, reserved_bytes: 0 },
            FieldConfig { enabled: true, reserved_bytes: 32 },
            FieldConfig { enabled: true, reserved_bytes: 32 },
            FieldConfig { enabled: true, reserved_bytes: 32 },
            FieldConfig { enabled: true, reserved_bytes: 32 },
            FieldConfig { enabled: true, reserved_bytes: 32 },
            FieldConfig { enabled: true, reserved_bytes: 32 },
        ];
        self.transfer_with_config(&defaults)
    }

    fn transfer_with_config(&self, field_configs: &[FieldConfig]) -> Vec<u8> {
        let mut board_area = Vec::new();

        // Board Area Header
        board_area.push(0x01); // Format version
        board_area.push(0x00); // Area lenght
        board_area.push(0x00); // Language code ( 0 for English )

        // Parse Mfg Date Time
        let mfg_time = if field_configs[0].enabled {
            parse_mfg_time(&self.board_mfg_date_time)
        } else {
            0
        };

        board_area.push((mfg_time & 0xFF) as u8);
        board_area.push(((mfg_time >> 8) & 0xFF) as u8);
        board_area.push(((mfg_time >> 16) & 0xFF) as u8);

        let encode_field = |field: &str, config: &FieldConfig| -> Option<Vec<u8>> {
            if !config.enabled {
                return None;
            }
            let mut bytes = field.as_bytes().to_vec();
            if config.reserved_bytes > 0 && bytes.len() < config.reserved_bytes {
                bytes.resize(config.reserved_bytes, b' ');
            }
            let len = bytes.len().min(0x3F);
            let mut res = vec![0xC0 | len as u8];
            res.extend_from_slice(&bytes[..len]);
            Some(res)
        };

        if let Some(f) = encode_field(&self.board_manufacturer, &field_configs[1]) { board_area.extend(f); }
        if let Some(f) = encode_field(&self.board_product_name, &field_configs[2]) { board_area.extend(f); }
        if let Some(f) = encode_field(&self.board_serial_number, &field_configs[3]) { board_area.extend(f); }
        if let Some(f) = encode_field(&self.board_part_number, &field_configs[4]) { board_area.extend(f); }
        if let Some(f) = encode_field(&self.board_fru_file_id, &field_configs[5]) { board_area.extend(f); }
        if let Some(f) = encode_field(&self.board_extra, &field_configs[6]) { board_area.extend(f); }

        board_area.push(0xC1);
        board_area.push(0x00); // Checksum placeholder

        // fill up the rest area space with 8 Byte
        while (board_area.len() % 8) != 0 {
            board_area.push(0x00);
        }

        // Update Area length
        board_area[1] = (board_area.len() / 8) as u8;

        // Update checksum
        let checksum = (0x100u16 - (board_area.iter().map(|&b| b as u16).sum::<u16>() % 256)) % 256; // Calculate checksum
        if let Some(last_byte) = board_area.last_mut() {
            *last_byte = checksum as u8;
        }

        board_area
    }
}
