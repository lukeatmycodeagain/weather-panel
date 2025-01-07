use std::{
    env,
    net::{IpAddr, Ipv4Addr},
};

// shared data models
pub mod models;
pub use models::Weather;
pub use models::WeatherQuery;

pub fn ip_configuration() -> IpAddr {
    let is_container = get_env_var("IS_CONTAINER", false);

    let address: IpAddr = if is_container {
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)) // Bind to all interfaces (0.0.0.0)
    } else {
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) // Bind to localhost (127.0.0.1)
    };

    address
}

// generic for getting environment variables
pub fn get_env_var<T: std::str::FromStr + ToString>(key: &str, default_value: T) -> T {
    env::var(key)
        .unwrap_or_else(|_| default_value.to_string()) // Get the variable as a string
        .parse::<T>()  // Try parsing it to the correct type
        .unwrap_or(default_value)  // If parsing fails, return the default value
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // Helper function to set and clear environment variables for testing
    fn set_env_var(key: &str, value: &str) {
        env::set_var(key, value);
    }

    fn clear_env_var(key: &str) {
        env::remove_var(key);
    }

    #[test]
    fn test_get_u16_env_var_existing() {
        set_env_var("PORT", "8080");

        let port: u16 = get_env_var("PORT", 3000);

        assert_eq!(port, 8080);

        clear_env_var("PORT");
    }

    #[test]
    fn test_get_u16_env_var_missing() {
        clear_env_var("PORT");

        let port: u16 = get_env_var("PORT", 3000);

        assert_eq!(port, 3000); // Default value is used

        clear_env_var("PORT");
    }

    #[test]
    fn test_get_u16_env_var_invalid() {
        set_env_var("PORT", "invalid");

        let port: u16 = get_env_var("PORT", 3000);

        assert_eq!(port, 3000); // Default value is used due to parse failure

        clear_env_var("PORT");
    }

    #[test]
    fn test_get_string_env_var_existing() {
        set_env_var("API_URL", "https://example.com");

        let api_url: String = get_env_var("API_URL", "http://default.com".to_string());

        assert_eq!(api_url, "https://example.com");

        clear_env_var("API_URL");
    }

    #[test]
    fn test_get_string_env_var_missing() {
        clear_env_var("API_URL");

        let api_url: String = get_env_var("API_URL", "http://default.com".to_string());

        assert_eq!(api_url, "http://default.com"); // Default value is used

        clear_env_var("API_URL");
    }

    #[test]
    fn test_get_string_env_var_invalid() {
        set_env_var("API_URL", "Not a valid URL");

        let api_url: String = get_env_var("API_URL", "http://default.com".to_string());

        assert_eq!(api_url, "Not a valid URL"); // Value is returned as is even if it's "invalid"

        clear_env_var("API_URL");
    }

    // Testing for boolean values
    #[test]
    fn test_get_bool_env_var_existing() {
        set_env_var("DEBUG", "true");

        let debug: bool = get_env_var("DEBUG", false);

        assert_eq!(debug, true);

        clear_env_var("DEBUG");
    }

    #[test]
    fn test_get_bool_env_var_missing() {
        clear_env_var("DEBUG");

        let debug: bool = get_env_var("DEBUG", false);

        assert_eq!(debug, false); // Default value is used

        clear_env_var("DEBUG");
    }

    #[test]
    fn test_get_bool_env_var_invalid() {
        set_env_var("DEBUG", "notabool");

        let debug: bool = get_env_var("DEBUG", false);

        assert_eq!(debug, false); // Default value is used due to parse failure

        clear_env_var("DEBUG");
    }
}
