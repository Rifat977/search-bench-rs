use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub brand: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub availability: String,
    #[serde(default)]
    pub reviews_count: String,
    #[serde(default)]
    pub rating: String,
    #[serde(default)]
    pub discount: String,
    #[serde(default)]
    pub manufacturer: String,
    #[serde(default)]
    pub category: String,
}

