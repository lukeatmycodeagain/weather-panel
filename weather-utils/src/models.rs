use rocket::form::{self, Error};
use rocket::FromForm;
use serde::{Deserialize, Serialize};


#[derive(FromForm, Debug, Serialize, Deserialize)]
pub struct WeatherQuery {
    #[field(validate = validate_latitude())]
    pub latitude: f64,
    #[field(validate= validate_longitude())]
    pub longitude: f64,
}

fn validate_latitude<'v>(lat: &f64) -> form::Result<'v, ()> {
    if *lat < -90.0 || *lat > 90.0 {
        Err(Error::validation("latitude must be between -90.0 and 90.0").into())
    } else {
        Ok(())
    }
}

fn validate_longitude<'v>(long: &f64) -> form::Result<'v, ()> {
    if *long < -180.0 || *long > 180.0 {
        Err(Error::validation("longitude must be between -180.0 and 180.0").into())
    } else {
        Ok(())
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Weather {
    pub time: String,
    pub temperature: f64,
    pub location_name: String,
}
