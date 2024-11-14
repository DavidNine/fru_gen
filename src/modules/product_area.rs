use serde::Deserialize;
use super::area::Area;


#[derive(Debug, Deserialize)]
pub struct Product {
    pub product_manufacturer: String,
    pub product_product_name: String,
    pub product_part_number: String,
    pub product_version: String,
    pub product_serial_number: String,
    pub product_asset_tag: String,   
    pub product_extra: String
}


impl Product {
    pub fn new(
        product_manufacturer: String,
        product_product_name: String,
        product_part_number: String,
        product_version: String,
        product_serial_number: String,
        product_asset_tag: String,
        product_extra: String,
    ) -> Self {
        Product {
            product_manufacturer,
            product_product_name,
            product_part_number,
            product_version,
            product_serial_number,
            product_asset_tag,
            product_extra,
        }
    }
    pub fn print_all(&self) {
        println!("Product Manufacturer   = {}" ,&self.product_manufacturer);
        println!("Product Name exceed    = {}" ,&self.product_product_name);
        println!("Product Part Number    = {}" ,&self.product_part_number);
        println!("Product Version exceed = {}" ,&self.product_version);
        println!("Product Serial Number  = {}" ,&self.product_serial_number);
        println!("Product Asset Tag      = {}" ,&self.product_asset_tag);
        println!("Product Extra          = {}" ,&self.product_extra);
    }
}

impl Area for Product {

    fn check_area_length(&self, field_name: &str, field_value: &str) {
        if field_value.len() > 0x3F {
            panic!("Error: String length of {} exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", field_name, field_value.len());
        }
    }

    fn validate(&self) {
        self.check_area_length("Product Manufacturer" ,&self.product_manufacturer);
        self.check_area_length("Product Name exceed" ,&self.product_product_name);
        self.check_area_length("Product Part Number" ,&self.product_part_number);
        self.check_area_length("Product Version exceed" ,&self.product_version);
        self.check_area_length("Product Serial Number" ,&self.product_serial_number);
        self.check_area_length("Product Asset Tag" ,&self.product_asset_tag);
        self.check_area_length("Product Extra" ,&self.product_extra);
    }



    fn transfer_as_byte(&self) -> Vec<u8> {
    
        self.validate();
        let mut product_area = Vec::new();
        
        // Product Area Header
        product_area.push(0x01);  // Format version
        product_area.push(0x00);  // Area lenght
        
        
        product_area.push(0xC0 | self.product_manufacturer.len() as u8);
        product_area.extend_from_slice(self.product_manufacturer.as_bytes());
        
        
        product_area.push(0xC0 | self.product_product_name.len() as u8);
        product_area.extend_from_slice(self.product_product_name.as_bytes());
        

        product_area.push(0xC0 | self.product_part_number.len() as u8);
        product_area.extend_from_slice(self.product_part_number.as_bytes());
        
        
        product_area.push(0xC0 | self.product_version.len() as u8);
        product_area.extend_from_slice(self.product_version.as_bytes());
        
        
        product_area.push(0xC0 | self.product_serial_number.len() as u8);
        product_area.extend_from_slice(self.product_serial_number.as_bytes());
        
        
        product_area.push(0xC0 | self.product_asset_tag.len() as u8);
        product_area.extend_from_slice(self.product_asset_tag.as_bytes());
        

        product_area.push(0xC0 | self.product_extra.len() as u8);
        product_area.extend_from_slice(self.product_extra.as_bytes());
        

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
