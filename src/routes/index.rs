use std::collections::HashMap;
use std::sync::Arc;

use rocket::{get, post, Request, State};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use serde_json::json;
use crate::utils::AppState;
use crate::database::models::*;
use crate::database::utils::password_utils::{hash_password, verify_password};
use crate::utils::auth::generate_jwt;

#[get("/")]
pub fn index() -> Json<serde_json::Value> {
    Json(json!({"Author":"Sugat Bagde"}))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupedProduct {
    pub title: String,
    pub slug: String,
    pub desc: String,
    pub img: String,
    pub category: String,
    pub color: Vec<String>,
    pub size: Vec<String>,
    pub price: f32,
    pub availableQty: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WrappedProducts {
    pub products: HashMap<String, GroupedProduct>,
}

#[get("/getproducts")]
pub async fn get_products(state: &State<Arc<AppState>>) -> Json<WrappedProducts> {
    let products: Vec<Product> = Product::get_all(&state.db).await;

    let mut grouped_products: HashMap<String, GroupedProduct> = HashMap::new();

    for product in products {
        if product.stock_qty == 0 {
            continue;
        }

        let entry = grouped_products.entry(product.title.clone()).or_insert_with(|| GroupedProduct {
            title: product.title.clone(),
            slug: product.slug.clone(),
            desc: product.desc.clone(),
            img: product.img.clone(),
            category: product.category.clone(),
            color: vec![product.color.clone()],
            size: vec![product.size.clone()],
            price: product.price,
            availableQty: product.stock_qty,
        });

        if !entry.color.contains(&product.color) {
            entry.color.push(product.color);
        }
        if !entry.size.contains(&product.size) {
            entry.size.push(product.size);
        }
    }

    Json(WrappedProducts { products: grouped_products })
}

#[derive(Debug, Deserialize)]
pub struct UserCredentials {
    pub name: String,
    pub email: String,
    pub password: String
}

#[post("/signup", format = "application/json", data = "<credentials>")]
pub async fn sign_up(credentials: Json<UserCredentials>, state: &State<Arc<AppState>>) -> Json<serde_json::Value> {
    let user = User {
        name : credentials.name.clone(),
        email : credentials.email.clone(),
        password_hash : hash_password( credentials.password.as_str()).expect("Error"),
        is_admin : false,
        id : None
    };

    println!("User is : {:?}", user);

    match user.save(&state.db).await {
        Ok(user) => {
            Json(json!({"success" : true , "message" : "Success", "email" : user.email }))
        },
        Err(e) => {
            println!("{:?}",e);
            Json(json!({ "error" : "There was problem Creating Account" }))
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String
}

#[post("/login", format = "application/json", data = "<credentials>")]
pub async fn login(credentials : Json<LoginCredentials>, state: &State<Arc<AppState>>) -> Json<serde_json::Value> {
    let user_res = User::find_by_email(credentials.email.as_str(), &state.db).await;

    match user_res {
        Ok(user) => {
            if verify_password(&user.password_hash, credentials.password.as_str()).expect("Something went wrong User hash verify") {
                let token = generate_jwt(&user, &state.jwt_key_pair, &state.db).await;
                Json(json!({"success" : true, "message": "Yeh! Logged in Successfully!", "token" : token }))
            }
            else {
                Json(json!({"success" : false, "error" : "Invalid Credentials" }))
            }
        },
        Err(surrealdb::Error::Api(surrealdb::error::Api::Query(_))) => {
            Json(json!({"success" : false, "error" : "Invalid Credentials" }))
        },
        Err(_) => {
            Json(json!({"success" : false, "error": "Unable to retrieve data" }))
        }
    }
}


// todo : Write this route properly
use rocket::http::Status;
use rocket::request::{Outcome, FromRequest};

struct JwtToken<'r>(&'r str);

#[derive(Debug)]
pub enum JwtError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtToken<'r> {
    type Error = JwtError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        fn is_valid(token: &str) -> bool {
            token == "valid_api_key"
        }

        match req.headers().get_one("x-api-key") {
            None => Outcome::Error((Status::BadRequest, JwtError::Missing)),
            Some(token) if is_valid(token) => Outcome::Success(JwtToken(token)),
            Some(_) => Outcome::Error((Status::BadRequest, JwtError::Invalid)),
        }
    }
}

#[get("/verify-user")]
pub async fn verify_user(token: JwtToken<'_>, state: &State<Arc<AppState>>) -> Json<serde_json::Value> {
    todo!()
}

