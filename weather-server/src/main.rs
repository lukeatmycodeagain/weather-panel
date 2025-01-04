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
    let endpoint = microservice_endpoint(Microservice::Weather);
    
    println!("Requesting weather from {endpoint}");

    // Make a request to the microservice's endpoint
    let client = reqwest::Client::new();
    let response = client.get(&endpoint).send().await;

    match response {
        Ok(res) if res.status().is_success() => {
            let body = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to parse response".to_string());
            println!("Body: {body}");
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

fn microservice_endpoint(service: Microservice) -> String {
    match service {
        Microservice::Weather => weather_endpoint(),
        Microservice::_NotImplemented => default_endpoint(),
    }
}

fn weather_endpoint() -> String {
    let address = weather_utils::ip_configuration();
    // Set the port using the WEATHER_MICROSERVICE_PORT environment variable, defaulting to 8080 if not set
    let port = weather_utils::port_from_env("WEATHER_MICROSERVICE_PORT", 8080);
    let endpoint = weather_utils::endpoint_from_env("WEATHER_MICROSERVICE_URL", format!("{address}:{port}"));
    endpoint
}

fn default_endpoint() -> String {
    format!("{}:{}",IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8000)
}
