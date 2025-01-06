#[macro_use]
extern crate rocket;

use dotenvy::dotenv;
use reqwest;
use std::net::{IpAddr, Ipv4Addr};
use weather_utils::Weather;
use weather_utils::WeatherQuery;

use rocket::form::{Contextual, Form};
use rocket::fs::{relative, FileServer, Options};
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::{get, launch, post, routes, uri};
use rocket_dyn_templates::{context, Template};

enum Microservice {
    Weather,
    _NotImplemented,
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    let (address, port) = server_config();
    rocket::build()
        .attach(Template::fairing())
        .mount(
            "/public",
            FileServer::new(
                relative!("/public"),
                Options::Missing | Options::NormalizeDirs,
            ),
        )
        .mount("/", routes![root, weather, create_weather_query, display_weather])
        //.mount("/weather", routes![weather, create_weather_query]) // TODO: Figure out why this doesn't
        .register("/", catchers![not_found])
        .configure(rocket::Config {
            address,
            port,
            ..Default::default()
        })
}

#[get("/")]
async fn root() -> Template {
    Template::render("root", context! { message: "Hello, from Luke"})
}

#[catch(404)]
fn not_found() -> &'static str {
    "Page not found"
}

/* #[get("/favicon.ico")]
fn favicon() -> rocket::response::NamedFile {
    rocket::response::NamedFile::open("favicon/favicon.ico").unwrap() // Adjust the path accordingly
} */

#[get("/weather")]
async fn weather() -> Template {
    let endpoint = microservice_endpoint(Microservice::Weather);

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
    Template::render("weather", context! { title: "Weather"})
}

#[post("/weather", data = "<form>")]
async fn create_weather_query(form: Form<Contextual<'_, WeatherQuery>>) -> Result<Flash<Redirect>, Template> {
    if let Some(ref query) = form.value {
        println!("Query connected data successful {} {}", query.latitude, query.longitude);
        let redirect_url = uri!(display_weather(query.latitude, query.longitude));
        print!("Redirecting to: {redirect_url}");
        let message = Flash::success(
            Redirect::to(redirect_url),
            "It Worked",
        );
        return Ok(message);
    } 
    else {
        println!("Form wasn't valid");
    }

    let error_messages: Vec<String> = form
        .context
        .errors()
        .map(|error| {
            let name = error.name.as_ref().unwrap().to_string();
            let description = error.to_string();
            format!("'{}' {}", name, description)
        })
        .collect();

    Err(Template::render(
        "weather",
        context! {
            latitude : form.context.field_value("latitude"),
            longitude : form.context.field_value("longitude"),
            latitude_error : form.context.field_errors("latitude").count() > 0,
            longitude_error : form.context.field_errors("longitude").count() > 0,
            errors: error_messages
        },
    ))
}

#[get("/weather?<lat>&<long>")]
async fn display_weather(lat: f64, long: f64, flash: Option<FlashMessage<'_>>) -> Template {
    println!("Received coordinates: Latitude: {}, Longitude: {}", lat, long);
    let message = flash.map_or_else(|| String::default(), |msg| msg.message().to_string());
    Template::render("weather_view", context! { lat , long, message })
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
    let address =
        weather_utils::endpoint_from_env("WEATHER_MICROSERVICE_URL", format!("http://localhost"));
    let endpoint = format!("{address}:{port}");
    endpoint
}

fn default_endpoint() -> String {
    format!("{}:{}", IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8000)
}
