use serde::Deserialize;
use super::area::{Area, FieldConfig};

#[derive(Debug, Deserialize)]
pub struct Chassis {
    pub chassis_type: u8,
    pub chassis_part_number: String,
    pub chassis_serial_number: String,
    pub chassis_extra: String,
}

impl Chassis {
    pub fn new(
        chassis_type: u8,
        chassis_part_number: String,
        chassis_serial_number: String,
        chassis_extra: String,
    ) -> Self {
        Chassis {
            chassis_type,
            chassis_part_number,
            chassis_serial_number,
            chassis_extra,
        }
    }
    pub fn print_all(&self) {
        println!("Chassis Part Number   = {}", &self.chassis_part_number);
        println!("Chassis Serial Number = {}", &self.chassis_serial_number);
        println!("Chassis Extra         = {}", &self.chassis_extra);
    }
}

impl Area for Chassis {
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
        self.check_area_length("Chassis Part Number", &self.chassis_part_number);
        self.check_area_length("Chassis Serial Number", &self.chassis_serial_number);
        self.check_area_length("Chassis Extra", &self.chassis_extra);
    }

    fn transfer_as_byte(&self) -> Vec<u8> {
        let defaults = vec![
            FieldConfig { enabled: true, reserved_bytes: 0 },
            FieldConfig { enabled: true, reserved_bytes: 32 },
            FieldConfig { enabled: true, reserved_bytes: 32 },
            FieldConfig { enabled: true, reserved_bytes: 32 },
        ];
        self.transfer_with_config(&defaults)
    }

    fn transfer_with_config(&self, field_configs: &[FieldConfig]) -> Vec<u8> {
        let mut chassis_area = Vec::new();

        // Chassis area header
        chassis_area.push(0x01); // Format version
        chassis_area.push(0x00); // Area length
        chassis_area.push(0x00); // Chassis type (to be set)

        // Chassis type
        if field_configs[0].enabled {
            chassis_area[2] = self.chassis_type;
        }

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

        if let Some(f) = encode_field(&self.chassis_part_number, &field_configs[1]) {
            chassis_area.extend(f);
        }
        if let Some(f) = encode_field(&self.chassis_serial_number, &field_configs[2]) {
            chassis_area.extend(f);
        }
        if let Some(f) = encode_field(&self.chassis_extra, &field_configs[3]) {
            chassis_area.extend(f);
        }

        // End of Chassis area, 0xC1 as end Byte
        chassis_area.push(0xC1);
        chassis_area.push(0x00); // Checksum placeholder

        // fill up the rest area space with 8 Byte
        while (chassis_area.len() % 8) != 0 {
            chassis_area.push(0x00);
        }

        // Update Area length
        chassis_area[1] = (chassis_area.len() / 8) as u8;

        // Update checksum
        let checksum =
            (0x100u16 - (chassis_area.iter().map(|&b| b as u16).sum::<u16>() % 256)) % 256; // Calculate checksum
        if let Some(last_byte) = chassis_area.last_mut() {
            *last_byte = checksum as u8;
        }

        chassis_area
    }
}
