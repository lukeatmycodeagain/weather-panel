#[macro_use]
extern crate rocket;

use dotenvy::dotenv;
use reqwest;
use rocket::form::{Contextual, Form};
use rocket::fs::{relative, FileServer, Options};
use rocket::response::{Flash, Redirect};
use rocket::{get, launch, post, routes, uri};
use rocket_dyn_templates::{context, Template};
use serde_json;
use std::net::{IpAddr, Ipv4Addr};
use weather_utils::Weather;
use weather_utils::WeatherQuery;

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
        .mount(
            "/",
            routes![root, weather, create_weather_query, display_weather],
        )
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
    Template::render("root", context! { message: "My Projects"})
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
    Template::render(
        "weather",
        context! { title: "Weather", longitude: "", longitude_error:"", latitude:"", latitude_error: "", lat: "", long: "", message: ""},
    )
}

#[post("/weather", data = "<form>")]
async fn create_weather_query(
    form: Form<Contextual<'_, WeatherQuery>>,
) -> Result<Flash<Redirect>, Template> {
    if let Some(ref query) = form.value {
        let redirect_url = uri!(display_weather(query.latitude, query.longitude));
        let message = Flash::success(Redirect::to(redirect_url), "It Worked");
        return Ok(message);
    } else {
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
            title: "Weather",
            latitude : form.context.field_value("latitude"),
            longitude : form.context.field_value("longitude"),
            latitude_error : form.context.field_errors("latitude").count() > 0,
            longitude_error : form.context.field_errors("longitude").count() > 0,
            errors: error_messages,
            lat: "",
            long: "",
            weather: {},
            message: "",
        },
    ))
}

#[get("/weather?<lat>&<long>")]
async fn display_weather(lat: f64, long: f64) -> Template {
    let endpoint = microservice_endpoint(Microservice::Weather);

    // back to a query object for easy serialization, not optimal, but quickly accommplished
    let weather_query = WeatherQuery {
        latitude: lat,
        longitude: long,
    };
    let json_body = serde_json::to_string(&weather_query).unwrap();

    // Make a request to the microservice's endpoint
    let client = reqwest::Client::new();
    let response = client
        .post(&endpoint) // Use POST to deliver json to handler in the microservice
        .header("Content-Type", "application/json")
        .body(json_body)
        .send()
        .await;

    match response {
        Ok(res) if res.status().is_success() => {
            let body = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to parse response".to_string());
            let weather: Result<Weather, _> = serde_json::from_str(&body);
            match weather {
                Ok(weather_data) => Template::render(
                    "weather",
                    context! {
                        title: "Weather",
                        longitude: "",
                        longitude_error:"",
                        latitude:"",
                        latitude_error: "",
                        lat: lat,
                        long: long,
                        weather: weather_data,
                        message: "Successful!"
                    },
                ),
                Err(_) => Template::render(
                    "weather",
                    context! {
                        title: "Weather",
                        longitude: "",
                        longitude_error: "",
                        latitude: "",
                        latitude_error: "",
                        lat: lat,
                        long: long,
                        weather: {},
                        message: "Failed to parse weather data as JSON"
                    },
                ),
            }
        }
        _ => {
            println!("got a bad request back!!");

            Template::render(
                "weather",
                context! {
                    title: "Weather",
                    longitude: "",
                    longitude_error: "",
                    latitude: "",
                    latitude_error: "",
                    lat: lat,
                    long: long,
                    weather: {},
                    message: "Unsuccessful response"
                },
            )
        }
    }
}

fn server_config() -> (IpAddr, u16) {
    let address = weather_utils::ip_configuration();
    // Set the port using the ROCKET_PORT environment variable, defaulting to 8000 if not set
    let port: u16 = weather_utils::get_env_var("ROCKET_PORT", 8000);
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
    let port: u16 = weather_utils::get_env_var("WEATHER_MICROSERVICE_PORT", 8080);
    let address: String =
        weather_utils::get_env_var("WEATHER_MICROSERVICE_URL", "http://localhost".to_string());
    let endpoint = format!("{address}:{port}");
    endpoint
}

fn default_endpoint() -> String {
    format!("{}:{}", IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8000)
}
