use super::area::{Area, FieldConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Product {
    pub product_manufacturer: String,
    pub product_product_name: String,
    pub product_part_number: String,
    pub product_version: String,
    pub product_serial_number: String,
    pub product_asset_tag: String,
    pub product_fru_file_id: String,
    pub product_extra: String,
}

impl Product {
    pub fn new(
        product_manufacturer: String,
        product_product_name: String,
        product_part_number: String,
        product_version: String,
        product_serial_number: String,
        product_asset_tag: String,
        product_fru_file_id: String,
        product_extra: String,
    ) -> Self {
        Product {
            product_manufacturer,
            product_product_name,
            product_part_number,
            product_version,
            product_serial_number,
            product_asset_tag,
            product_fru_file_id,
            product_extra,
        }
    }
    pub fn print_all(&self) {
        println!("Product Manufacturer   = {}", &self.product_manufacturer);
        println!("Product Name           = {}", &self.product_product_name);
        println!("Product Part Number    = {}", &self.product_part_number);
        println!("Product Version        = {}", &self.product_version);
        println!("Product Serial Number  = {}", &self.product_serial_number);
        println!("Product Asset Tag      = {}", &self.product_asset_tag);
        println!("Product Fru ID         = {}", &self.product_fru_file_id);
        println!("Product Extra          = {}", &self.product_extra);
    }
}

impl Area for Product {
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
        self.check_area_length("Product Manufacturer", &self.product_manufacturer);
        self.check_area_length("Product Name", &self.product_product_name);
        self.check_area_length("Product Part Number", &self.product_part_number);
        self.check_area_length("Product Version", &self.product_version);
        self.check_area_length("Product Serial Number", &self.product_serial_number);
        self.check_area_length("Product Asset Tag", &self.product_asset_tag);
        self.check_area_length("Product Fru ID", &self.product_fru_file_id);
        self.check_area_length("Product Extra", &self.product_extra);
    }

    fn transfer_as_byte(&self) -> Vec<u8> {
        let defaults = vec![
            FieldConfig { enabled: true, reserved_bytes: 32 },
            FieldConfig { enabled: true, reserved_bytes: 32 },
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
        let mut product_area = Vec::new();

        // Product Area Header
        product_area.push(0x01); // Format version
        product_area.push(0x00); // Area length
        product_area.push(0x00); // Language code ( 0 for English )

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

        if let Some(f) = encode_field(&self.product_manufacturer, &field_configs[0]) { product_area.extend(f); }
        if let Some(f) = encode_field(&self.product_product_name, &field_configs[1]) { product_area.extend(f); }
        if let Some(f) = encode_field(&self.product_part_number, &field_configs[2]) { product_area.extend(f); }
        if let Some(f) = encode_field(&self.product_version, &field_configs[3]) { product_area.extend(f); }
        if let Some(f) = encode_field(&self.product_serial_number, &field_configs[4]) { product_area.extend(f); }
        if let Some(f) = encode_field(&self.product_asset_tag, &field_configs[5]) { product_area.extend(f); }
        if let Some(f) = encode_field(&self.product_fru_file_id, &field_configs[6]) { product_area.extend(f); }
        if let Some(f) = encode_field(&self.product_extra, &field_configs[7]) { product_area.extend(f); }

        product_area.push(0xC1);
        product_area.push(0x00); // Checksum placeholder

        // fill up the rest area space with 8 Byte
        while (product_area.len() % 8) != 0 {
            product_area.push(0x00);
        }

        // Update Area length
        product_area[1] = (product_area.len() / 8) as u8;

        // Update checksum
        let checksum =
            (0x100u16 - (product_area.iter().map(|&b| b as u16).sum::<u16>() % 256)) % 256; // Calculate checksum
        if let Some(last_byte) = product_area.last_mut() {
            *last_byte = checksum as u8;
        }

        product_area
    }
}
