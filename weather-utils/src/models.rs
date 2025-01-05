
use rocket::{FromForm};

#[derive(FromForm, Debug)]
pub struct Person {
    #[field(validate=len(1..))]
    pub(crate) first_name: String,
    #[field(validate=len(1..))]
    pub(crate) last_name: String,
}


use serde::{Serialize, Deserialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct Weather{
    pub time: String,
    pub temperature: f64
}
