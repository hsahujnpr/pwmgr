use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Credential {
    pub username: String,
    pub password: String,
}
