mod models;
mod tantivy_search;

use actix_web::{get, web, App, HttpServer, Responder, HttpResponse};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::models::Product;
use crate::tantivy_search::SearchEngine;

use std::time::Instant;
use serde::Serialize;
use log::{info, error};


#[derive(Serialize)]
struct SearchResponse {
    took_s: f64,
    results: Vec<Product>,
}

#[get("/search")]
async fn search(
    query: web::Query<std::collections::HashMap<String, String>>,
    engine: web::Data<Arc<Mutex<SearchEngine>>>
) -> impl Responder {
    
    let q = query.get("q").cloned().unwrap_or_default();
    info!("Search request received for query: '{}'", q);
    
    let engine = engine.lock().await;

    let start = Instant::now();

    let result = engine.search(&q, 10);

    let duration = start.elapsed();
    info!("Search completed in {:?}", duration);

    match result {
        Ok(products) => {
            info!("Search successful, found {} products", products.len());
            let response = SearchResponse {
                took_s: duration.as_secs_f64(),
                results: products,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("Search failed: {}", e);
            HttpResponse::InternalServerError().body(format!("Search failed: {}", e))
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    info!("Starting search-bench server...");

    let products = load_products("dataset/product.csv").expect("Failed to load dataset");
    info!("Loaded {} products from dataset", products.len());

    let engine = SearchEngine::new().expect("Failed to create search engine");
    engine.index_products(&products).expect("Index failed");
    info!("Search engine indexed successfully");

    let shared_engine = Arc::new(Mutex::new(engine));

    info!("Server running at http://127.0.0.1:8000");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_engine.clone()))
            .service(search)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}

fn load_products(path: &str) -> Result<Vec<Product>, Box<dyn std::error::Error>> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut products = Vec::new();
    for result in rdr.deserialize() {
        let record: Product = result?;
        products.push(record);
    }
    Ok(products)
}
