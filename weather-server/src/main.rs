use reqwest;
use std::net::{IpAddr, Ipv4Addr};

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
    let address = weather_utils::ip_configuration();
    // Set the port using the ROCKET_PORT environment variable, defaulting to 8000 if not set
    let port = weather_utils::port_from_env("ROCKET_PORT", 8000);
    (address, port)
}

fn get_microservice_endpoint(service: Microservice) -> (IpAddr, u16) {
    match service {
        Microservice::Weather => get_weather_endpoint(),
        Microservice::_NotImplemented => default_endpoint(),
    }
}

fn get_weather_endpoint() -> (IpAddr, u16) {
    let address = weather_utils::ip_configuration();
    // Set the port using the WEATHER_MICROSERVICE_PORT environment variable, defaulting to 8080 if not set
    let port = weather_utils::port_from_env("WEATHER_MICROSERVICE_PORT", 8080);
    (address, port)
}

fn default_endpoint() -> (IpAddr, u16) {
    (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8000)
}
