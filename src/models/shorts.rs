use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewShortRequest {
    pub long_url: String,
}
