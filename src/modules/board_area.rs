use core::panic;

use serde::Deserialize;
use super::area::Area;


#[derive(Debug, Deserialize)]
pub struct Board {
    pub board_manufacturer: String,
    pub board_product_name: String,
    pub board_serial_number: String,
    pub board_part_number: String,
    pub board_fru_file_id: String,
    pub board_extra: String,
}

impl Board {
    pub fn new(
        board_manufacturer: String,
        board_product_name: String,
        board_serial_number: String,
        board_part_number: String,
        board_fru_file_id: String,
        board_extra: String,
    ) -> Self {
        Board {
            board_manufacturer,
            board_product_name,
            board_serial_number,
            board_part_number,
            board_fru_file_id,
            board_extra,
        }
    }
}

impl Area for Board{
    
    fn check_area_length(&self, field_name: &str, field_value: &str) {
        if field_value.len() > 0x3F {
            panic!("Error: String length of {} exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", field_name, field_value.len());
        }
    }

    fn validate(&self) {
        self.check_area_length("Board Manufacturer" ,&self.board_manufacturer);
        self.check_area_length("Board Product Name" ,&self.board_product_name);
        self.check_area_length("Board Serial Number",&self.board_serial_number);
        self.check_area_length("Board Part Number"  ,&self.board_part_number);
        self.check_area_length("Board Part Number"  ,&self.board_fru_file_id);
        self.check_area_length("Board Extra"        ,&self.board_extra);
    }

    fn transfer_as_byte(&self) -> Vec<u8> {
        
        self.validate();
        let mut board_area = Vec::new();
        
        // Board Area Header
        board_area.push(0x01);  // Format version
        board_area.push(0x00);  // Area lenght
        board_area.push(0x00);  // Language code ( 0 for English )
        board_area.push(0x00);  // Board Mfg. Date/Time ( 0 for unspecified )
        board_area.push(0x00);  // Board Mfg. Date/Time ( 0 for unspecified )
        board_area.push(0x00);  // Board Mfg. Date/Time ( 0 for unspecified )
        
        
        board_area.push(0xC0 | self.board_manufacturer.len() as u8);
        board_area.extend_from_slice(self.board_manufacturer.as_bytes());
        
        
        board_area.push(0xC0 | self.board_product_name.len() as u8);
        board_area.extend_from_slice(self.board_product_name.as_bytes());
        
        
        board_area.push(0xC0 | self.board_serial_number.len() as u8);
        board_area.extend_from_slice(self.board_serial_number.as_bytes());
        
        
        board_area.push(0xC0 | self.board_part_number.len() as u8);
        board_area.extend_from_slice(self.board_part_number.as_bytes());
        
        
        board_area.push(0xC0 | self.board_fru_file_id.len() as u8);
        board_area.extend_from_slice(self.board_fru_file_id.as_bytes());
        

        board_area.push(0xC0 | self.board_extra.len() as u8);
        board_area.extend_from_slice(self.board_extra.as_bytes());
        

        board_area.push(0xC1);
        
        // fill up the rest area space with 8 Byte
        while (board_area.len() % 8) != 0{
            board_area.push(0x00);
        }    
        
        // Update Area length
        board_area[1] = (board_area.len() / 8 ) as u8;      
        
        // Update checksum
        let checksum = (0x100u16 - (board_area.iter().map(|&b| b as u16).sum::<u16>() % 256)) % 256;  // Calculate checksum
        if let Some(last_byte) = board_area.last_mut() {
            *last_byte = checksum as u8;
        }
        
        board_area
    }
}