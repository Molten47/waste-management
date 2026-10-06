use std::io;

#[derive(Debug)]
enum District {
    Ikeja,
    Surulere,
    EtiOsa,
}

struct Truck {
    id: String,
    driver: String,
    manager: String,
    district: District,
    street: String,
    first_house: u32,
    last_house: u32,
    shift_start: u8,
    shift_end: u8,
    lane: u8,
}

#[derive(Debug)]
enum TruckError {
    InvalidHouseRange,
    InvalidShift,
    HourOutOfRange,
    DuplicateId,
    InvalidLane,
}

fn collection_day(district: &District) -> &'static str {
    match district {
        District::Ikeja => "Monday",
        District::Surulere => "Tuesday",
        District::EtiOsa => "Wednesday",
    }
}

fn find_truck<'a>(fleet: &'a [Truck], street: &str, house: u32) -> Option<&'a Truck> {
    for truck in fleet {
        let on_street = truck.street.eq_ignore_ascii_case(street);
        let in_range = (truck.first_house..=truck.last_house).contains(&house);

        if on_street && in_range {
            return Some(truck);
        }
    }

    None
}

fn shift_length(truck: &Truck) -> u8 {
   truck.shift_end - truck.shift_start
}

fn add_truck(fleet: &mut Vec<Truck>, truck: Truck) -> Result<(), TruckError> {
    if truck.first_house > truck.last_house {
        return Err(TruckError::InvalidHouseRange);
    }

    if truck.shift_start > 23 || truck.shift_end > 24 {
        return Err(TruckError::HourOutOfRange);
    }

    if truck.shift_end <= truck.shift_start {
        return Err(TruckError::InvalidShift);
    }

    if fleet.iter().any(|t| t.id == truck.id) {
        return Err(TruckError::DuplicateId);
    }
    if truck.lane > 0 {
        return  Err(TruckError::InvalidLane);
    }

    fleet.push(truck);
    Ok(())
}

fn main() {
    let  mut fleet = vec![
        Truck {
            id: "TT-0023".to_string(),
            driver: "Chinedu Okafor".to_string(),
            manager: "Mrs. Ngozi Eze".to_string(),
            district: District::Ikeja,
            street: "Allen Avenue".to_string(),
            first_house: 1,
            last_house: 50,
            shift_start: 6,
            shift_end: 14,
            lane:2
        },
        Truck {
            id: "TT-0034".to_string(),
            driver: "Aisha Bello".to_string(),
            manager: "Mrs. Ngozi Eze".to_string(),
            district: District::Ikeja,
            street: "Allen Avenue".to_string(),
            first_house: 51,
            last_house: 100,
            shift_start: 6,
            shift_end: 14,
            lane:2
        },
        Truck {
            id: "TT-0041".to_string(),
            driver: "Tunde Adeyemi".to_string(),
            manager: "Mr. Emeka Nwosu".to_string(),
            district: District::Surulere,
            street: "Adeniran Ogunsanya Street".to_string(),
            first_house: 1,
            last_house: 80,
            shift_start: 8,
            shift_end: 16,
            lane:3
        },
        Truck{
            id:"TT-00432".to_string(),
            driver:"Chigozie Roland".to_string(),
            district: District::EtiOsa,
            manager:"AddulWahab Akinola".to_string(),
            street: "Adetokumbo Odetola Street, VI".to_string(),
            first_house:1,
            last_house: 50,
            shift_start:13,
            shift_end: 17,
            lane:9

        }
    ];

    let new_truck = Truck {
        id: "TT-0050".to_string(),
        driver: "Funke Adebayo".to_string(),
        manager: "Mrs. Ngozi Eze".to_string(),
        district: District::Ikeja,
        street: "Allen Avenue".to_string(),
        first_house: 101,
        last_house: 150,
        shift_start: 6,
        shift_end: 14,
        lane: 2,
    };

    match add_truck(&mut fleet, new_truck) {
        Ok(()) => println!("Truck added."),
        Err(error) => println!("Could not add truck: {error:?}"),
    }

    loop {
        println!("Enter street (or 'quit'):");
        let mut street_input = String::new();
        io::stdin().read_line(&mut street_input).unwrap();
        let street = street_input.trim();

        if street.eq_ignore_ascii_case("quit") {
            break;
        }

        println!("Enter house number:");
        let mut house_input = String::new();
        io::stdin().read_line(&mut house_input).unwrap();

        let house = match house_input.trim().parse::<u32>() {
            Ok(number) => number,
            Err(error) => {
                println!("Invalid house number: {error}");
                continue;
            }
        };

   match find_truck(&fleet, street, house) {
    Some(truck) => println!(
        "{} | {} | {:?} | {} | shift {}:00-{}:00 ({} hrs)",
        truck.id,
        truck.driver,
        truck.district,
        collection_day(&truck.district),
        truck.shift_start,
        truck.shift_end,
        shift_length(truck)
    ),
    None => println!("No truck covers house {house} on {street}."),
}
        
    }

}