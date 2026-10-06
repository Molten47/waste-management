use std::io;

use crate::errors::TruckError;
use crate::logic;
use crate::models::Truck;

fn prompt(message: &str) -> Result<String, TruckError> {
    println!("{message}");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_number(message: &str) -> Result<u32, TruckError> {
    let text = prompt(message)?;
    let number = text.parse::<u32>()?;
    Ok(number)
}

pub fn run(fleet: &[Truck]) -> Result<(), TruckError> {
    loop {
        let street = prompt("Enter street (or 'quit'):")?;
        if street.eq_ignore_ascii_case("quit") {
            break;
        }

        let house = match prompt_number("Enter house number:") {
            Ok(number) => number,
            Err(TruckError::InvalidNumber(error)) => {
                println!("Invalid house number: {error}");
                continue;
            }
            Err(other) => return Err(other),
        };

        match logic::find_truck(fleet, &street, house) {
            Some(truck) => println!(
                "{} | {} (mgr: {}) | {:?} | lane {} | {} | shift {}:00-{}:00 ({} hrs)",
                truck.id,
                truck.driver,
                truck.manager,
                truck.district,
                truck.lane,
                truck.district.collection_day(),
                truck.shift_start,
                truck.shift_end,
                truck.shift_length()
            ),
            None => println!("No truck covers house {house} on {street}."),
        }
    }

    Ok(())
}