use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::{Error, RecordId, Surreal};
use surrealdb::Error::Api;
use surrealdb::error::Api::Query;

use super::super::models::{DatabaseIO};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Option<RecordId>,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    #[serde(default)]
    pub is_admin: bool
}

impl DatabaseIO for User{
    type Model = User;

    fn table_name() -> &'static str {
        "User"
    }

    async fn init(db: &Surreal<Client>) -> Result<(), Error> {
        let query_str = r#"
        DEFINE TABLE IF NOT EXISTS User SCHEMAFULL;

        DEFINE FIELD IF NOT EXISTS email ON TABLE User
            ASSERT string::is::email($value)
            PERMISSIONS FOR select, create WHERE true 
            PERMISSIONS FOR update, delete WHERE false;
        
        DEFINE FIELD IF NOT EXISTS name ON TABLE User TYPE STRING
            PERMISSIONS FOR select, create, update WHERE true;
            
        DEFINE FIELD IF NOT EXISTS password_hash ON TABLE User TYPE STRING PERMISSIONS FULL;
        DEFINE FIELD IF NOT EXISTS is_admin ON TABLE User TYPE BOOL DEFAULT false PERMISSIONS FOR update WHERE false;
        
        DEFINE INDEX IF NOT EXISTS emailIndex ON TABLE User FIELDS email UNIQUE;
        "#;

        let resp = db.query(query_str).await;

        match resp {
            Ok(_) => {
                println!("Users Table Initialized...");
                Ok(())
            },
            Err(e) => {
                println!("Users DB Error : {:?}",e);

                Err(e)
            }
        }
    }

    async fn get_all(db: &Surreal<Client>) -> Vec<Self::Model> {
        let query = db.query("SELECT * FROM User").await;
        let mut response = query.ok().unwrap();
        let all_users = response.take(0).
            ok().unwrap();
        all_users
    }

    async fn save(self, db: &Surreal<Client>) -> Result<Self::Model, Error> {
        match self.id.clone() {
            None => {
                let users : Option<User> = db.create("User").content(self.clone()).await?;
                users.ok_or(Api(Query("Failed to create user".to_string())))
            }
            Some(id) => {
                let user :Option<User> = db.update(id).content(self.clone()).await?;
                user.ok_or(Api(Query("Failed to update user".to_string())))
            }
        }
    }
}

impl User {
    pub async fn find_by_email(email: &str, db: &Surreal<Client>) -> Result<User, Error> {
        let query = db.query("SELECT * FROM User WHERE email = $email")
            .bind(("email", email.to_string()))
            .await;
        let mut response = query.ok().unwrap();
        let mut user: Vec<User> = response.take(0).
            ok().unwrap();
        user.pop().ok_or(Api(Query("User not found".to_string())))
    }
}