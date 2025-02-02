#[macro_use] extern crate rocket;

use std::sync::Arc;

use hackerwear_api::database::db::{Credentials, connect_to_database};
use hackerwear_api::database::models::*;
use hackerwear_api::utils::AppState;
use hackerwear_api::routes::index::*;

#[rocket::main]
async fn main() -> Result<(), String> {
    let credentials = Credentials {
        namespace : "<namespace>",
        username: "<username>",
        password: "<password>",
        database : "<Database>"
    };

    // Initializing Database Connection
    println!("Initialising surrealdb...");
    let db = connect_to_database("<surrealdb server hostname>", 
                                 credentials)
        .await
        .expect("Could not connect to database");

    // Init Models
    Product::init(&db).await.expect("Could not initialize product table");
    User::init(&db).await.expect("Could not initialize user table");
    
    println!("Rocket ready to Launch....\nIn 3... 2... 1...");
    rocket::build()
        .mount("/", routes![index, get_products, sign_up, login, verify_user])
        .manage(Arc::new(AppState::new(db) ))
        .launch().await
        .expect("Could not launch app");

    Ok(())
}