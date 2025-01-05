#[macro_use]
extern crate rocket;

use reqwest;
use std::net::{IpAddr, Ipv4Addr};
use dotenvy::dotenv;
use weather_utils;
use rocket::fs::NamedFile;

enum Microservice {
    Weather,
    _NotImplemented,
}

#[catch(404)]
fn not_found() -> &'static str {
    "Page not found"
}

/* #[get("/favicon.ico")]
fn favicon() -> rocket::response::NamedFile {
    rocket::response::NamedFile::open("favicon/favicon.ico").unwrap() // Adjust the path accordingly
} */

#[get("/")]
fn index() -> &'static str {
    "Hello, luke!"
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
    dotenv().ok();
    // Check if the app is running inside a container using the IS_CONTAINER environment variable
    let (address, port) = server_config();
    println!("Binding to {}:{}", address, port);
    rocket::build()
        .mount("/", routes![index, weather])
        .register("/", catchers![not_found])
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
    // Set the port using the WEATHER_MICROSERVICE_PORT environment variable, defaulting to 8080 if not set
    let port = weather_utils::port_from_env("WEATHER_MICROSERVICE_PORT", 8080);
    println!("The weather endpoint port is {port}");
    let address = weather_utils::endpoint_from_env("WEATHER_MICROSERVICE_URL", format!("http://localhost"));
    println!("The weather endpoint address is {address}");
    let endpoint = format!("{address}:{port}");
    endpoint
}

fn default_endpoint() -> String {
    format!("{}:{}",IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8000)
}
