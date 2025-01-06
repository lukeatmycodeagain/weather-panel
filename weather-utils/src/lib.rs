use std::{
    env,
    net::{IpAddr, Ipv4Addr},
};

// shared data models
pub mod models;
pub use models::Weather;
pub use models::WeatherQuery;

pub fn ip_configuration() -> IpAddr {
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

pub fn port_from_env(key: &str, default_port: u16) -> u16 {
    env::var(key)
        .unwrap_or_else(|_| default_port.to_string())
        .parse::<u16>()
        .unwrap_or(default_port)
}

pub fn endpoint_from_env(key: &str, default_endpoint: String) -> String {
    env::var(key).unwrap_or_else(|_| default_endpoint)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
