#[derive(Debug, Clone, PartialEq)]
pub enum District {
    Ikeja,
    Surulere,
    EtiOsa,
}

impl District {
    pub fn collection_day(&self) -> &'static str {
        match self {
            District::Ikeja => "Monday",
            District::Surulere => "Tuesday",
            District::EtiOsa => "Wednesday",
        }
    }
}

pub struct Truck {
    pub id: String,
    pub driver: String,
    pub manager: String,
    pub district: District,
    pub street: String,
    pub first_house: u32,
    pub last_house: u32,
    pub shift_start: u8,
    pub shift_end: u8,
    pub lane: u8,
}

impl Truck {
    /// Hours on shift, including overnight shifts (e.g. 22 -> 6 = 8 hrs).
    pub fn shift_length(&self) -> u8 {
        (self.shift_end + 24 - self.shift_start) % 24
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct RouteInfo {
    pub fleet_code: String,
    pub driver: Option<String>,
    pub supervisor: Option<String>,
    pub district: String,
    pub collection_day: String,
    pub street: String,
    pub lane: i16,
    pub first_house: i32,
    pub last_house: i32,
    pub shift_start: i16,
    pub shift_end: i16,
}