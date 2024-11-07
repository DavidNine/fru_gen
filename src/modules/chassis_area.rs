use serde::Deserialize;
use super::fru_data::FruData;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Chassis {
    pub chassis_type: String,
    pub chassis_part_number: String,
    pub chassis_serial_number:String,
}


impl Chassis {
    pub fn build(fru_data: &FruData) -> Vec<u8>{
        let mut chassis_area = Vec::new();

        // Chassis area header
        chassis_area.push(0x01);    // Format version
        chassis_area.push(0x00);    // Area length
        chassis_area.push(0x00);    // Chassis type (to be set)
        
        
        // Chassis type
        let chassis_type_code = transfer_chassis_type_str_to_code(&fru_data.chassis.chassis_type).unwrap_or(0x02);  // If string not found, default will be Unknow.
        chassis_area[2] = chassis_type_code;
        
        // Chassis Part Number
        chassis_area.push(0xC0 | fru_data.chassis.chassis_part_number.len() as u8);         // Chassis Part Number length
        if fru_data.chassis.chassis_part_number.len() > 0x3F {
            panic!("Error: String length of Chassis Part Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.chassis.chassis_part_number.len());
        }
        chassis_area.extend_from_slice(&fru_data.chassis.chassis_part_number.as_bytes());   // Chassis Part Number data
        
        
        // Chassis Serial Number
        chassis_area.push(0xC0 | fru_data.chassis.chassis_serial_number.len() as u8);       // Chassis Serial Number length
        if fru_data.chassis.chassis_serial_number.len() > 0x3F {
            panic!("Error: String length of Chassis Serial Number exceed limitation\nExp:[0x3F], Act:[0x{:02X}]", fru_data.chassis.chassis_serial_number.len());
        }
        chassis_area.extend_from_slice(&fru_data.chassis.chassis_serial_number.as_bytes()); // Chassis Serial Number data
        
        
        // End of Chassis area, 0xC1 as end Byte
        chassis_area.push(0xC1);
        
        
        // fill up the rest area space with 8 Byte
        while (chassis_area.len() % 8) != 0{
            chassis_area.push(0x00);
        }    
        
        // Update Area length
        chassis_area[1] = (chassis_area.len() / 8 ) as u8;      
        
        // Update checksum
        let checksum = (0x100u16 - (chassis_area.iter().map(|&b| b as u16).sum::<u16>() % 256)) % 256;  // Calculate checksum
        if let Some(last_byte) = chassis_area.last_mut() {
            *last_byte = checksum as u8;
        }
        
        chassis_area    // return chassis data. (Dtype = Vec<u8>)
    }
}




pub
fn transfer_chassis_type_str_to_code(chassis_type_str: &str) -> Option<u8>{
    let chassis_type_map = HashMap::from([
        ("Other"                , 0x01),
        ("Unknown"              , 0x02),
        ("Desktop"              , 0x03),
        ("Low Profile Desktop"  , 0x04),
        ("Pizza Box"            , 0x05),
        ("Mini Tower"           , 0x06),
        ("Tower"                , 0x07),
        ("Portable"             , 0x08),
        ("Laptop"               , 0x09),
        ("Notebook"             , 0x0A),
        ("Lunch Box"            , 0x10),
        ("Main Server Chassis"  , 0x11),
        ("Expansion Chassis"    , 0x12),
        ("SubChassis"           , 0x13),
        ("Bus Expansion Chassis", 0x14),
        ("Peripheral Chassis"   , 0x15),
        ("RAID Chassis"         , 0x16),
        ("Rack Mount Chassis"   , 0x17),
        ("Sealed-case PC"       , 0x18),
        ("Multi-system Chassis" , 0x19),
        ("Compact PCI"          , 0x1A),
        ("Advanced TCA"         , 0x1B),
        ("Blade"                , 0x1C),
        ("Blade Enclosure"      , 0x1D),
        ("Tablet"               , 0x1E),
        ("Convertible"          , 0x1F),
        ("Detachable"           , 0x20),
        ("IoT Gateway"          , 0x21),
        ("Embedded PC"          , 0x22),
        ("Mini PC"              , 0x23),
        ("Stick PC"             , 0x24),
    ]);

    chassis_type_map.get(chassis_type_str).copied()
}