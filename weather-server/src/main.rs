use std::{
    env,
    net::{IpAddr, Ipv4Addr},
};

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, luke!"
}

#[get("/weather")]
fn weather() -> &'static str {
    "Hello, weather!"
}

#[launch]
fn rocket() -> _ {
    // Check if the app is running inside a container using the IS_CONTAINER environment variable
    let is_container = env::var("IS_CONTAINER")
        .unwrap_or_else(|_| "false".to_string()) // Default to "false" if not set
        .to_lowercase()
        == "true"; // Compare case-insensitively

    println!("IS_CONTAINER: {}", is_container); // Debugging
    let address: IpAddr = if is_container {
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)) // Bind to all interfaces (0.0.0.0)
    } else {
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) // Bind to localhost (127.0.0.1)
    };
    // Set the port using the ROCKET_PORT environment variable, defaulting to 8000 if not set
    let port = env::var("ROCKET_PORT")
        .unwrap_or_else(|_| "8000".to_string()) // Default to "8000" if not set
        .parse::<u16>()
        .unwrap_or(8000); // Default to 8000 if parsing fails

    println!("Binding to {}:{}", address, port);

    rocket::build()
        .mount("/", routes![index, weather])
        .mount("/api", routes![weather])
        .configure(rocket::Config {
            address,
            port,
            ..Default::default()
        })
}
