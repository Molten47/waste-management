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

/* TESTS FOR MODELS.RS */

#[cfg(test)]
pub fn sample_truck(id: &str) -> Truck {
    Truck {
        id: id.to_string(),
        driver: "Test Driver".to_string(),
        manager: "Test Manager".to_string(),
        district: District::Ikeja,
        street: "Allen Avenue".to_string(),
        first_house: 1,
        last_house: 50,
        shift_start: 6,
        shift_end: 14,
        lane: 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_shift_length() {
        let truck = sample_truck("TT-0001");
        assert_eq!(truck.shift_length(), 8);
    }

    #[test]
    fn overnight_shift_length() {
        let mut truck = sample_truck("TT-0001");
        truck.shift_start = 22;
        truck.shift_end = 6;
        assert_eq!(truck.shift_length(), 8, "22:00 to 06:00 should be 8 hours");
    }

    #[test]
    fn districts_have_collection_days() {
        assert_eq!(District::Ikeja.collection_day(), "Monday");
        assert_eq!(District::Surulere.collection_day(), "Tuesday");
        assert_eq!(District::EtiOsa.collection_day(), "Wednesday");
    }

    #[test]
    fn district_equality() {
        assert_eq!(District::Ikeja, District::Ikeja);
        assert_ne!(District::Ikeja, District::Surulere);
    }
}
