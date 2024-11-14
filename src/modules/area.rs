pub trait Area {
    fn transfer_as_byte(&self) -> Vec<u8>;
    fn check_area_length(&self, field_name: &str, field_value: &str);
    fn validate(&self);
}