use crate::errors::TruckError;
use crate::models::Truck;

pub fn find_truck<'a>(fleet: &'a [Truck], street: &str, house: u32) -> Option<&'a Truck> {
    fleet.iter().find(|truck| {
        truck.street.eq_ignore_ascii_case(street)
            && (truck.first_house..=truck.last_house).contains(&house)
    })
}

pub fn add_truck(fleet: &mut Vec<Truck>, truck: Truck) -> Result<(), TruckError> {
    if truck.first_house > truck.last_house {
        return Err(TruckError::InvalidHouseRange);
    }
    if truck.shift_start > 23 || truck.shift_end > 23 {
        return Err(TruckError::HourOutOfRange);
    }
    if truck.shift_start == truck.shift_end {
        return Err(TruckError::InvalidShift);
    }
    if fleet.iter().any(|t| t.id == truck.id) {
        return Err(TruckError::DuplicateId);
    }

    fleet.push(truck);
    Ok(())
}