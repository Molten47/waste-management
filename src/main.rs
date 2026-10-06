mod cli;
mod data;
mod errors;
mod logic;
mod models;

use models::{District, Truck};

fn main() {
    let mut fleet = data::seed_fleet();

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

    match logic::add_truck(&mut fleet, new_truck) {
        Ok(()) => println!("Truck added."),
        Err(error) => println!("Could not add truck: {error}"),
    }

    if let Err(error) = cli::run(&fleet) {
        eprintln!("Fatal error: {error}");
    }
}