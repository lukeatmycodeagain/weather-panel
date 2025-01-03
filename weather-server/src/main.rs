use reqwest;
use std::{
    env,
    net::{IpAddr, Ipv4Addr},
};

use weather_utils;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, luke!"
}

enum Microservice {
    Weather,
    _NotImplemented,
}

#[get("/weather")]
async fn weather() -> Result<String, String> {
    let (address, port) = get_microservice_endpoint(Microservice::Weather);
    // Make a request to the microservice's weather endpoint
    let client = reqwest::Client::new();
    let url = format!("{}:{}", address, port); // Adjust the URL based on your Docker Compose setup
    let response = client.get(url).send().await;

    match response {
        Ok(res) if res.status().is_success() => {
            let body = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to parse response".to_string());
            Ok(body)
        }
        _ => Err("Failed to fetch weather data".to_string()),
    }
}

#[launch]
fn rocket() -> _ {
    // Check if the app is running inside a container using the IS_CONTAINER environment variable
    let (address, port) = server_config();
    println!("Binding to {}:{}", address, port);
    println!("Testing lib: {}", weather_utils::add(6, 36));
    rocket::build()
        .mount("/", routes![index, weather])
        .mount("/api", routes![weather])
        .configure(rocket::Config {
            address,
            port,
            ..Default::default()
        })
}

fn server_config() -> (IpAddr, u16) {
    let address = ip_configuration();
    // Set the port using the ROCKET_PORT environment variable, defaulting to 8000 if not set
    let port = port_from_env("ROCKET_PORT", 8000);
    (address, port)
}

fn ip_configuration() -> IpAddr {
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

    address
}

fn port_from_env(key: &str, default_port: u16) -> u16 {
    env::var(key)
        .unwrap_or_else(|_| default_port.to_string())
        .parse::<u16>()
        .unwrap_or(default_port)
}

fn get_microservice_endpoint(service: Microservice) -> (IpAddr, u16) {
    match service {
        Microservice::Weather => get_weather_endpoint(),
        Microservice::_NotImplemented => default_endpoint(),
    }
}

fn get_weather_endpoint() -> (IpAddr, u16) {
    let address = ip_configuration();
    // Set the port using the WEATHER_MICROSERVICE_PORT environment variable, defaulting to 8080 if not set
    let port = port_from_env("WEATHER_MICROSERVICE_PORT", 8080);
    (address, port)
}

fn default_endpoint() -> (IpAddr, u16) {
    (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8000)
}
