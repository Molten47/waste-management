use std::io;

use sqlx::PgPool;

use crate::data;
use crate::errors::TruckError;

fn prompt(message: &str) -> Result<String, TruckError> {
    println!("{message}");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_number(message: &str) -> Result<i32, TruckError> {
    let text = prompt(message)?;
    let number = text.parse::<i32>()?;
    Ok(number)
}

pub async fn run(pool: &PgPool) -> Result<(), TruckError> {
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

        match data::find_route(pool, &street, house).await? {
            Some(r) => println!(
                "{} | driver: {} | supervisor: {} | {} ({}) | lane {} | shift {}:00-{}:00",
                r.fleet_code,
                r.driver.as_deref().unwrap_or("Unassigned"),
                r.supervisor.as_deref().unwrap_or("Unassigned"),
                r.district,
                r.collection_day,
                r.lane,
                r.shift_start,
                r.shift_end
            ),
            None => println!("No truck covers house {house} on {street}."),
        }
    }

    Ok(())
}