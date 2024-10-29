use serde::Deserialize;

/*
    Define FRU data area struct
*/
#[derive(Debug, Deserialize)]
pub struct Chassis {
    pub chassis_type: String,
    pub chassis_part_number: String,
    pub chassis_serial_number:String,
}

#[derive(Debug, Deserialize)]
pub struct Board {
    pub board_manufacturer: String,
    pub board_product_name: String,
    pub board_serial_number: String,
    pub board_part_number: String,
    pub board_fru_file_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Product {
    pub product_manufacturer: String,
    pub product_product_name: String,
    pub product_part_number: String,
    pub product_version: String,
    pub product_serial_number: String,
    pub product_asset_tag: String,   
}

#[derive(Debug, Deserialize)]
pub struct FruData {
    pub chassis: Chassis,
    pub board: Board,
    pub product: Product,
}

