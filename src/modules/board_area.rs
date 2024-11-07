use serde::Deserialize;
use super::fru_data::FruData;


#[derive(Debug, Deserialize)]
pub struct Board {
    pub board_manufacturer: String,
    pub board_product_name: String,
    pub board_serial_number: String,
    pub board_part_number: String,
    pub board_fru_file_id: String,
}



    
impl Board {
    
    pub fn build(fru_data: &FruData) -> Vec<u8> {
        let mut board_area = Vec::new();
        
        // Board Area Header
        board_area.push(0x01);  // Format version
        board_area.push(0x00);  // Area lenght
        board_area.push(0x00);  // Language code ( 0 for English )
        board_area.push(0x00);  // Board Mfg. Date/Time ( 0 for unspecified )
        board_area.push(0x00);  // Board Mfg. Date/Time ( 0 for unspecified )
        board_area.push(0x00);  // Board Mfg. Date/Time ( 0 for unspecified )
        
        
        board_area.push(0xC0 | fru_data.board.board_manufacturer.len() as u8);
        if fru_data.board.board_manufacturer.len() > 0x3F {
            panic!("Error: String length of Board Manufacturer exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.board.board_manufacturer.len());
        }
        board_area.extend_from_slice(&fru_data.board.board_manufacturer.as_bytes());
        
        
        board_area.push(0xC0 | fru_data.board.board_product_name.len() as u8);
        if fru_data.board.board_product_name.len() > 0x3F {
            panic!("Error: String length of Board Product Name exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.board.board_product_name.len());
        }
        board_area.extend_from_slice(&fru_data.board.board_product_name.as_bytes());
        
        
        board_area.push(0xC0 | fru_data.board.board_serial_number.len() as u8);
        if fru_data.board.board_serial_number.len() > 0x3F {
            panic!("Error: String length of Board Serial Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.board.board_serial_number.len());
        }
        board_area.extend_from_slice(&fru_data.board.board_serial_number.as_bytes());
        
        
        board_area.push(0xC0 | fru_data.board.board_part_number.len() as u8);
        if fru_data.board.board_part_number.len() > 0x3F {
            panic!("Error: String length of Board Part Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.board.board_part_number.len());
        }
        board_area.extend_from_slice(&fru_data.board.board_part_number.as_bytes());
        
        
        board_area.push(0xC0 | fru_data.board.board_fru_file_id.len() as u8);
        if fru_data.board.board_fru_file_id.len() > 0x3F {
            panic!("Error: String length of Board Part Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.board.board_fru_file_id.len());
        }
        board_area.extend_from_slice(&fru_data.board.board_fru_file_id.as_bytes());
        
        
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