#[macro_use] extern crate rocket;

use std::sync::Arc;

use hackerwear_api::database::db::{connect_to_database};
use hackerwear_api::database::models::session_token::SessionToken;
use hackerwear_api::database::models::*;
use hackerwear_api::utils::{extract_app_config_from_env, AppConfig, AppState};
use hackerwear_api::routes::index::*;


#[rocket::main]
async fn main() -> Result<(), String> {
    let app_config : AppConfig = extract_app_config_from_env().expect("Unable to extract env data");

    // Initializing Database Connection
    println!("Initialising surrealdb...");
    let db = connect_to_database(&app_config.surreal_hostname, 
                                 app_config.credentials)
        .await
        .expect("Could not connect to database");

    // Init Models
    Product::init(&db).await.expect("Could not initialize product table");
    User::init(&db).await.expect("Could not initialize user table");
    SessionToken::init(&db).await.expect("Could not initialize Session Token table");
    
    println!("Rocket ready to Launch....\nIn 3... 2... 1...");
    rocket::build()
        .mount("/", routes![index, get_products, sign_up, login, verify_user])
        .manage(Arc::new(AppState::new(db, &app_config.jwt_key_path)))
        .launch().await
        .expect("Could not launch app");

    Ok(())
}