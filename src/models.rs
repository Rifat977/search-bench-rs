use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
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
    pub reviews_count: Option<i32>,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub discount: String,
    #[serde(default)]
    pub manufacturer: String,
    #[serde(default)]
    pub category: String,
}

