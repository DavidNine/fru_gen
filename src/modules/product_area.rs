use serde::Deserialize;
use super::fru_data::FruData;


#[derive(Debug, Deserialize)]
pub struct Product {
    pub product_manufacturer: String,
    pub product_product_name: String,
    pub product_part_number: String,
    pub product_version: String,
    pub product_serial_number: String,
    pub product_asset_tag: String,   
}



impl Product {

    pub fn build(fru_data: &FruData) -> Vec<u8> {
        let mut product_area = Vec::new();
        
        // Product Area Header
        product_area.push(0x01);  // Format version
        product_area.push(0x00);  // Area lenght
        
        
        product_area.push(0xC0 | fru_data.product.product_manufacturer.len() as u8);
        if fru_data.product.product_manufacturer.len() > 0x3F {
            panic!("Error: String length of Product Manufacturer exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.product.product_manufacturer.len());
        }
        product_area.extend_from_slice(&fru_data.product.product_manufacturer.as_bytes());
        
        
        
        product_area.push(0xC0 | fru_data.product.product_product_name.len() as u8);
        if fru_data.product.product_product_name.len() > 0x3F {
            panic!("Error: String length of Product Name exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.product.product_product_name.len());
        }
        product_area.extend_from_slice(&fru_data.product.product_product_name.as_bytes());
        
        
        product_area.push(0xC0 | fru_data.product.product_part_number.len() as u8);
        if fru_data.product.product_part_number.len() > 0x3F {
            panic!("Error: String length of Product Part Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.product.product_part_number.len());
        }
        product_area.extend_from_slice(&fru_data.product.product_part_number.as_bytes());
        
        
        product_area.push(0xC0 | fru_data.product.product_version.len() as u8);
        if fru_data.product.product_version.len() > 0x3F {
            panic!("Error: String length of Product Version exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.product.product_version.len());
        }
        product_area.extend_from_slice(&fru_data.product.product_version.as_bytes());
        
        
        product_area.push(0xC0 | fru_data.product.product_serial_number.len() as u8);
        if fru_data.product.product_serial_number.len() > 0x3F {
            panic!("Error: String length of Product Serial Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.product.product_serial_number.len());
        }
        product_area.extend_from_slice(&fru_data.product.product_serial_number.as_bytes());
        
        
        product_area.push(0xC0 | fru_data.product.product_asset_tag.len() as u8);
        if fru_data.product.product_asset_tag.len() > 0x3F {
            panic!("Error: String length of Product Asset Tag exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.product.product_asset_tag.len());
        }
        product_area.extend_from_slice(&fru_data.product.product_asset_tag.as_bytes());
        
        
        product_area.push(0xC1);
        
        // fill up the rest area space with 8 Byte
        while (product_area.len() % 8) != 0{
            product_area.push(0x00);
        }    
        
        // Update Area length
        product_area[1] = (product_area.len() / 8 ) as u8;      
        
        // Update checksum
        let checksum = (0x100u16 - (product_area.iter().map(|&b| b as u16).sum::<u16>() % 256)) % 256;  // Calculate checksum
        if let Some(last_byte) = product_area.last_mut() {
            *last_byte = checksum as u8;
        }
        
        product_area
        
    }
}
