use serde::Deserialize;
use super::area::Area;

#[derive(Debug, Deserialize)]
pub struct Internal {
    pub internal_info: String,
}

impl Internal {
    pub fn new(internal_info: String) -> Self {
        Internal {
            internal_info
        }
    }
}

impl Area for Internal {


    fn check_area_length(&self, field_name: &str, field_value: &str) {
        if field_value.len() > 0x3F {
            panic!("Error: String length of {} exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", field_name, field_value.len());
        }
    }

    fn validate(&self) {
        self.check_area_length("Internal information" ,&self.internal_info);
    }

    fn transfer_as_byte(&self) -> Vec<u8> {
        self.validate();
        let mut internal_use_area = Vec::new();

        internal_use_area.push(0x01);   // Version Code.
        
        while internal_use_area.len() % 8 != 0 {
            internal_use_area.push(0x00);
        }


        let checksum = (0x100u16 - (internal_use_area.iter().map(|&i| i as u16).sum::<u16>() %256)) % 256;
        if let Some(last_byte) = internal_use_area.last_mut() {
            *last_byte = checksum as u8;
        }

        internal_use_area
    }
}