#[macro_use]
extern crate rocket;

use dotenvy::dotenv;
use reqwest;
use std::net::{IpAddr, Ipv4Addr};
use weather_utils::Person;
use weather_utils::Weather;

use rocket::{get, launch, post, routes, uri};
use rocket::form::{Contextual, Form};
use rocket::fs::{FileServer, Options, relative};
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket_dyn_templates::{context, Template};

enum Microservice {
    Weather,
    _NotImplemented,
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    // Check if the app is running inside a container using the IS_CONTAINER environment variable
    let (address, port) = server_config();
    println!("Binding to {}:{}", address, port);
    rocket::build()
        .attach(Template::fairing())
        .mount(
            "/public",
            FileServer::new(
                relative!("/public"),
                Options::Missing | Options::NormalizeDirs,
            ),
        )
        .mount("/", routes![root, create, hello, weather])
        .register("/", catchers![not_found])
        .mount("/api", routes![weather])
        .configure(rocket::Config {
            address,
            port,
            ..Default::default()
        })
}
#[get("/")]
async fn root() -> Template {
    Template::render("root", context! { message: "Hello, Rust"})
}

#[catch(404)]
fn not_found() -> &'static str {
    "Page not found"
}

/* #[get("/favicon.ico")]
fn favicon() -> rocket::response::NamedFile {
    rocket::response::NamedFile::open("favicon/favicon.ico").unwrap() // Adjust the path accordingly
} */

/* #[get("/")]
fn index() -> &'static str {
    "Hello, luke!"
} */

#[get("/weather")]
async fn weather() -> Template {
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
        }
        _ => print!("Failed to fetch weather data"),
    }
    Template::render("weather", context! { message: "Hello, Rust"})
}

#[get("/hi?<name>")]
async fn hello(name: String, flash: Option<FlashMessage<'_>>) -> Template {
    let message = flash.map_or_else(|| String::default(), |msg| msg.message().to_string());
    Template::render("hello", context! { name , message })
}

#[post("/", data = "<form>")]
async fn create(form: Form<Contextual<'_, Person>>) -> Result<Flash<Redirect>, Template> {
    if let Some(ref person) = form.value {
        let name = format!("{} {}", person.first_name, person.last_name);
        let message = Flash::success(Redirect::to(uri!(hello(name))), "It Worked");
        return Ok(message);
    }

    let error_messages: Vec<String> = form.context.errors().map(|error| {
        let name = error.name.as_ref().unwrap().to_string();
        let description = error.to_string();
        format!("'{}' {}", name, description)
    }).collect();

    Err(Template::render("root", context! {
        first_name : form.context.field_value("first_name"),
        last_name : form.context.field_value("last_name"),
        first_name_error : form.context.field_errors("first_name").count() > 0,
        last_name_error : form.context.field_errors("last_name").count() > 0,
        errors: error_messages
    }))
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
    let address =
        weather_utils::endpoint_from_env("WEATHER_MICROSERVICE_URL", format!("http://localhost"));
    println!("The weather endpoint address is {address}");
    let endpoint = format!("{address}:{port}");
    endpoint
}

fn default_endpoint() -> String {
    format!("{}:{}", IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8000)
}
