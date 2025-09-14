mod models;
mod tantivy_search;

use actix_web::{get, web, App, HttpServer, Responder, HttpResponse};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::models::Product;
use crate::tantivy_search::SearchEngine;

use std::time::Instant;
use serde::Serialize;


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
    let engine = engine.lock().await;

    let start = Instant::now();

    let result = engine.search(&q, 10);

    let duration = start.elapsed();

 

    

    match result {
        Ok(products) => {
            let response = SearchResponse {
                took_s: duration.as_secs_f64(),
                results: products,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Search failed: {}", e)),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let products = load_products("dataset/product.csv").expect("Fialed to load dataset");

    let engine = SearchEngine::new().expect("Failed to create search engine");
    engine.index_products(&products).expect("Index failed");

    let shared_engine = Arc::new(Mutex::new(engine));

    println!("Server running at http://127.0.0.1:8000");

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
