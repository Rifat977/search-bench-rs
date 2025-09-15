use crate::models::Product;
use tantivy::schema::*;
use tantivy::{Document, Index};

pub struct SearchEngine {
    pub index: Index,
    pub schema: Schema,
}

impl SearchEngine {
    pub fn new() -> tantivy::Result<Self> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("brand", TEXT | STORED);
        schema_builder.add_text_field("description", TEXT | STORED);
        schema_builder.add_f64_field("price", STORED);
        schema_builder.add_text_field("currency", STORED);
        schema_builder.add_text_field("availability", STORED);
        schema_builder.add_i64_field("reviews_count", STORED);
        schema_builder.add_f64_field("rating", STORED);
        schema_builder.add_text_field("discount", STORED);
        schema_builder.add_text_field("manufacturer", STORED);
        schema_builder.add_text_field("category", STORED);

        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());

        Ok(Self { index, schema })
    }

    pub fn index_products(&self, products: &[Product]) -> tantivy::Result<()> {
        let mut writer = self.index.writer(50_000_000)?;
        let title = self.schema.get_field("title").unwrap();
        let brand = self.schema.get_field("brand").unwrap();
        let description = self.schema.get_field("description").unwrap();
        let price = self.schema.get_field("price").unwrap();
        let currency = self.schema.get_field("currency").unwrap();
        let availability = self.schema.get_field("availability").unwrap();
        let reviews_count = self.schema.get_field("reviews_count").unwrap();
        let rating = self.schema.get_field("rating").unwrap();
        let discount = self.schema.get_field("discount").unwrap();
        let manufacturer = self.schema.get_field("manufacturer").unwrap();
        let category = self.schema.get_field("category").unwrap();

        for p in products {
            let mut doc = Document::default();
            doc.add_text(title, &p.title);
            doc.add_text(brand, &p.brand);
            doc.add_text(description, &p.description);

            if let Some(price_val) = p.price {
                doc.add_f64(price, price_val);
            }

            doc.add_text(currency, &p.currency);
            doc.add_text(availability, &p.availability);

            if let Some(reviews_count_val) = p.reviews_count {
                doc.add_i64(reviews_count, reviews_count_val as i64);
            }
            if let Some(rating_val) = p.rating {
                doc.add_f64(rating, rating_val);
            }
            doc.add_text(discount, &p.discount);

            doc.add_text(manufacturer, &p.manufacturer);
            doc.add_text(category, &p.category);

            writer.add_document(doc)?;
        }
        writer.commit()?;
        Ok(())
    }

    pub fn search(&self, query_str: &str, limit: usize) -> tantivy::Result<Vec<Product>> {
        use tantivy::collector::TopDocs;
        use tantivy::query::QueryParser;

        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let title = self.schema.get_field("title").unwrap();
        let description = self.schema.get_field("description").unwrap();
        let brand = self.schema.get_field("brand").unwrap();
        let price = self.schema.get_field("price").unwrap();
        let currency = self.schema.get_field("currency").unwrap();
        let availability = self.schema.get_field("availability").unwrap();
        let reviews_count = self.schema.get_field("reviews_count").unwrap();
        let rating = self.schema.get_field("rating").unwrap();
        let discount = self.schema.get_field("discount").unwrap();
        let manufacturer = self.schema.get_field("manufacturer").unwrap();
        let category = self.schema.get_field("category").unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![title, description, brand]);
        let query = query_parser.parse_query(query_str)?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        let mut results = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved = searcher.doc(doc_address)?;

            let product = Product {
                title: retrieved
                    .get_first(title)
                    .and_then(|f| f.as_text())
                    .unwrap_or("")
                    .to_string(),
                brand: retrieved
                    .get_first(brand)
                    .and_then(|f| f.as_text())
                    .unwrap_or("")
                    .to_string(),
                description: retrieved
                    .get_first(description)
                    .and_then(|f| f.as_text())
                    .unwrap_or("")
                    .to_string(),
                price: retrieved
                    .get_first(price)
                    .and_then(|f| f.as_f64())
                    .map(|v| v as f64),
                currency: retrieved
                    .get_first(currency)
                    .and_then(|f| f.as_text())
                    .unwrap_or("")
                    .to_string(),
                availability: retrieved
                    .get_first(availability)
                    .and_then(|f| f.as_text())
                    .unwrap_or("")
                    .to_string(),
                reviews_count: retrieved
                    .get_first(reviews_count)
                    .and_then(|f| f.as_i64())
                    .map(|v| v as i32),
                rating: retrieved
                    .get_first(rating)
                    .and_then(|f| f.as_f64()),
                discount: retrieved
                    .get_first(discount)
                    .and_then(|f| f.as_text())
                    .unwrap_or("")
                    .to_string(),
                manufacturer: retrieved
                    .get_first(manufacturer)
                    .and_then(|f| f.as_text())
                    .unwrap_or("")
                    .to_string(),
                category: retrieved
                    .get_first(category)
                    .and_then(|f| f.as_text())
                    .unwrap_or("")
                    .to_string(),
            };

            results.push(product);
        }

        Ok(results)
    }
}
