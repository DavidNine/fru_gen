use serde::Deserialize;
use super::chassis_area::Chassis;
use super::board_area::Board;
use super::product_area::Product;


#[derive(Debug, Deserialize)]
pub struct FruData {
    pub chassis: Chassis,
    pub board: Board,
    pub product: Product,
}

impl FruData {
        
    pub
    fn show_fru_data(&self) {
        println!("============================================================");
        println!(" * {:<30}: {}", "chassis type", self.chassis.chassis_type);
        println!(" * {:<30}: {}", "chassis part number", self.chassis.chassis_part_number);
        println!(" * {:<30}: {}", "chassis serial number", self.chassis.chassis_serial_number);
        println!(" * {:<30}: {}", "board manufacturer", self.board.board_manufacturer);
        println!(" * {:<30}: {}", "board product name", self.board.board_product_name);
        println!(" * {:<30}: {}", "board serial number", self.board.board_serial_number);
        println!(" * {:<30}: {}", "board part number", self.board.board_part_number);
        println!(" * {:<30}: {}", "board fru file id", self.board.board_fru_file_id);
        println!(" * {:<30}: {}", "product manufacturer", self.product.product_manufacturer);
        println!(" * {:<30}: {}", "product product name", self.product.product_product_name);
        println!(" * {:<30}: {}", "product part number", self.product.product_part_number);
        println!(" * {:<30}: {}", "product version", self.product.product_version);
        println!(" * {:<30}: {}", "product serial number", self.product.product_serial_number);
        println!(" * {:<30}: {}", "product asset tag", self.product.product_asset_tag);
        println!("============================================================");
    }

}