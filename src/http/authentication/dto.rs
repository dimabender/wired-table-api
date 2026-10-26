use garde::Validate;
use serde::Deserialize;

#[derive(Validate, Deserialize)]
pub struct RegisterRequest {
    #[garde(length(min = 3, max = 32))]
    pub username: String,
    #[garde(email, length(max = 254))]
    pub email: String,
    #[garde(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Validate, Deserialize)]
pub struct LoginRequest {
    #[garde(email, length(max = 254))]
    pub email: String,
    #[garde(length(min = 8, max = 128))]
    pub password: String,
}
