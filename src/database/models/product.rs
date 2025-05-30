use std::collections::HashMap;

use rocket::serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::{Error, RecordId, Surreal, sql::Value};
use surrealdb::Error::Api;
use surrealdb::error::Api::Query;
use super::super::models::{DatabaseIO};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product{
    #[serde(default)]
    pub id: Option<RecordId>,
    pub title: String,
    pub slug: String,
    pub desc: String,
    pub img: String,
    pub category: String,
    pub color: String,
    pub size: String,
    pub price: f32,
    pub stock_qty: u32,
    #[serde(default)]
    pub extras: Option<serde_json::Value>
}

impl DatabaseIO for Product{
    type Model = Product;

    fn table_name() -> &'static str {
        "Product"
    }

    async fn init(db: &Surreal<Client>) -> Result<(), Error> {
        let query_str = r#"
        DEFINE TABLE IF NOT EXISTS Product SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS title ON TABLE Product TYPE String PERMISSIONS FULL;
        DEFINE FIELD IF NOT EXISTS slug ON TABLE Product TYPE String PERMISSIONS FULL;
        DEFINE FIELD IF NOT EXISTS desc ON TABLE Product TYPE String PERMISSIONS FULL;
        DEFINE FIELD IF NOT EXISTS img ON TABLE Product TYPE String PERMISSIONS FULL;
        DEFINE FIELD IF NOT EXISTS category ON TABLE Product TYPE String PERMISSIONS FULL;
        DEFINE FIELD IF NOT EXISTS color ON TABLE Product TYPE String PERMISSIONS FULL;
        DEFINE FIELD IF NOT EXISTS size ON TABLE Product TYPE String PERMISSIONS FULL;
        DEFINE FIELD IF NOT EXISTS price ON TABLE Product TYPE Number PERMISSIONS FULL;
        DEFINE FIELD IF NOT EXISTS stock_qty ON TABLE Product TYPE Number PERMISSIONS FULL;
        DEFINE FIELD IF NOT EXISTS extras ON TABLE Product FLEXIBLE TYPE OBJECT DEFAULT {} PERMISSIONS FULL; // Json extra data

        DEFINE INDEX IF NOT EXISTS slugIndex ON TABLE Product FIELDS slug UNIQUE;"#;

        let resp = db.query(query_str).await;

        match resp {
            Ok(_) => {
                println!("Product Table Initialized...");
                Ok(())
            },
            Err(e) => {
                // dbg!(e);
                println!("Products DB Error : {:?}",e);

                Err(e)
            }
        }
    }

    async fn get_all(db: &Surreal<Client>) -> Vec<Self::Model> {
        let query = db.query("SELECT * FROM Product").await;
        let mut response = query.ok().unwrap();
        let all_products = response.take(0).
            ok().unwrap();
        all_products
    }

    async fn save(self, db: &Surreal<Client>) -> Result<Self::Model, Error> {
        match self.id.clone() {
            None => {
                let product : Option<Product> = db.create("Product").content(self.clone()).await?;
                product.ok_or(Api(Query("Failed to create product".to_string())))
            }
            Some(id) => {
                let product :Option<Product> = db.update(id).content(self.clone()).await?;
                product.ok_or(Api(Query("Failed to update product".to_string())))
            }
        }
    }
}


impl Product {
    pub async fn find(db: &Surreal<Client>, filters: HashMap<String, Value>) -> Result<Vec<Product>, Error> {
        // Build dynamic WHERE clause
        let mut where_clauses = Vec::new();
        for key in filters.keys() {
            where_clauses.push(format!("{key} = ${key}"));
        }

        let where_clause = if where_clauses.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Build and bind query dynamically
        let query = format!("SELECT * FROM Product {}", where_clause);
        let mut query_builder = db.query(&query);

        for (key, val) in filters {
            query_builder = query_builder.bind((key, val));
        }

        let mut response = query_builder.await?;
        let result: Vec<Product> = response.take(0)?;
        Ok(result)
    }
}