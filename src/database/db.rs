use surrealdb::{ Surreal, engine::remote::ws::{Client,Wss} };
use surrealdb::opt::auth::Namespace;

pub struct Credentials {
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

pub async fn connect_to_database<'a>(hostname : &'a str, credentials : Credentials) -> Result<Surreal<Client>, surrealdb::Error>{
    let db = Surreal::new::<Wss>(hostname).await?;

    db.signin(Namespace {
        namespace: &credentials.namespace,
        username: &credentials.username,
        password: &credentials.password,
    })
        .await?;

    db.use_db(credentials.database).await?;

    Ok(db)
}