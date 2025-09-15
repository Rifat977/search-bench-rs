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

use sqlx::SqlitePool;


#[derive(Serialize)]
struct SearchResponse {
    engine: String,
    query: String,
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
                engine: "tantivy".to_string(),
                query: q,
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

#[get("/sqlsearch")]
async fn sqlsearch(
    query: web::Query<std::collections::HashMap<String, String>>,
    pool: web::Data<SqlitePool>,
) -> impl Responder {
    let q = query.get("q").cloned().unwrap_or_default();
    let start = Instant::now();

    let sql_query = format!("SELECT title, brand, description, price, currency, availability, reviews_count, rating, discount, manufacturer, category FROM products WHERE title LIKE '%{}%' OR description LIKE '%{}%' OR brand LIKE '%{}%' LIMIT 10", q, q, q);

    let products = sqlx::query_as::<_, Product>(&sql_query)
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_else(|e| {
            error!("SQL query failed: {}", e);
            vec![]
        });

    let duration = start.elapsed();
    info!("SQL search completed in {:?}", duration);

    let response = SearchResponse {
        engine: "sql".to_string(),
        query: q,
        took_s: duration.as_secs_f64(),
        results: products,
    };

    HttpResponse::Ok().json(response)
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

    let pool = SqlitePool::connect("sqlite://products.db").await.expect("Failed to connect to database");
    
    insert_products_to_db(&pool, &products).await.expect("Failed to insert products to database");
    info!("Inserted {} products into database", products.len());
    
    let shared_pool = web::Data::new(pool);

    info!("Server running at http://127.0.0.1:8000");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_engine.clone()))
            .app_data(shared_pool.clone())
            .service(search)
            .service(sqlsearch)
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

async fn insert_products_to_db(pool: &SqlitePool, products: &[Product]) -> Result<(), sqlx::Error> {
    const BATCH_SIZE: usize = 1000;
    
    for chunk in products.chunks(BATCH_SIZE) {
        let mut tx = pool.begin().await?;
        
        for product in chunk {
            sqlx::query(
                r#"
                INSERT INTO products (title, brand, description, price, currency, availability, reviews_count, rating, discount, manufacturer, category)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&product.title)
            .bind(&product.brand)
            .bind(&product.description)
            .bind(product.price)
            .bind(&product.currency)
            .bind(&product.availability)
            .bind(product.reviews_count)
            .bind(product.rating)
            .bind(&product.discount)
            .bind(&product.manufacturer)
            .bind(&product.category)
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
    }
    
    Ok(())
}
