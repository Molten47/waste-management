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

/* TEST LOGIC.RS */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::sample_truck;

    #[test]
    fn finds_truck_for_house_in_range() {
        let fleet = vec![sample_truck("TT-0001")];
        let found = find_truck(&fleet, "Allen Avenue", 25);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "TT-0001");
    }

    #[test]
    fn street_match_ignores_case() {
        let fleet = vec![sample_truck("TT-0001")];
        assert!(find_truck(&fleet, "allen avenue", 25).is_some());
    }

    #[test]
    fn range_boundaries_are_inclusive() {
        let fleet = vec![sample_truck("TT-0001")]; // houses 1..=50
        assert!(find_truck(&fleet, "Allen Avenue", 1).is_some());
        assert!(find_truck(&fleet, "Allen Avenue", 50).is_some());
        assert!(find_truck(&fleet, "Allen Avenue", 0).is_none());
        assert!(find_truck(&fleet, "Allen Avenue", 51).is_none());
    }

    #[test]
    fn unknown_street_returns_none() {
        let fleet = vec![sample_truck("TT-0001")];
        assert!(find_truck(&fleet, "Nowhere Road", 25).is_none());
    }

    #[test]
    fn adds_valid_truck() -> Result<(), TruckError> {
        let mut fleet = Vec::new();
        add_truck(&mut fleet, sample_truck("TT-0001"))?;
        assert_eq!(fleet.len(), 1);
        Ok(())
    }

    #[test]
    fn rejects_reversed_house_range() {
        let mut fleet = Vec::new();
        let mut truck = sample_truck("TT-0001");
        truck.first_house = 90;
        truck.last_house = 10;

        let result = add_truck(&mut fleet, truck);
        assert!(matches!(result, Err(TruckError::InvalidHouseRange)));
        assert!(fleet.is_empty(), "a rejected truck must not be stored");
    }

    #[test]
    fn rejects_hours_above_23() {
        let mut fleet = Vec::new();
        let mut truck = sample_truck("TT-0001");
        truck.shift_end = 24;

        let result = add_truck(&mut fleet, truck);
        assert!(matches!(result, Err(TruckError::HourOutOfRange)));
    }

    #[test]
    fn rejects_same_start_and_end() {
        let mut fleet = Vec::new();
        let mut truck = sample_truck("TT-0001");
        truck.shift_start = 8;
        truck.shift_end = 8;

        let result = add_truck(&mut fleet, truck);
        assert!(matches!(result, Err(TruckError::InvalidShift)));
    }

    #[test]
    fn rejects_duplicate_id() {
        let mut fleet = vec![sample_truck("TT-0001")];
        let result = add_truck(&mut fleet, sample_truck("TT-0001"));
        assert!(matches!(result, Err(TruckError::DuplicateId)));
        assert_eq!(fleet.len(), 1);
    }
}
