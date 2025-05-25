use chrono::{TimeZone, Utc};
use rocket::serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::{Error, RecordId, Surreal};
use surrealdb::Error::Api;
use surrealdb::error::Api::Query;
use surrealdb::sql::{Datetime,Uuid};

use super::super::models::{DatabaseIO};
use super::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionToken {
    #[serde(default)]
    pub id: Option<RecordId>,
    pub jti: Uuid,           // UUID for the token
    pub user: RecordId,          // Assuming `User` is defined elsewhere
    pub issued_at: Datetime,      // Unix timestamp for when the token was issued
    pub expires_at: Datetime,     // Unix timestamp for when the token expires
    #[serde(default)]
    pub revoked: bool      // Indicates if the token is revoked
}

impl DatabaseIO for SessionToken{
    type Model = SessionToken;

    async fn init(db: &Surreal<Client>) -> Result<(), Error> {
        let query_str = r#"
      DEFINE TABLE IF NOT EXISTS sessiontoken SCHEMAFULL;

        DEFINE FIELD IF NOT EXISTS jti ON TABLE sessiontoken TYPE Uuid
            PERMISSIONS FOR select, create WHERE true 
            PERMISSIONS FOR update, delete WHERE false;
        
        DEFINE FIELD IF NOT EXISTS user ON TABLE sessiontoken TYPE record<User>
            PERMISSIONS FOR select, create WHERE true 
            PERMISSIONS FOR update, delete WHERE false;
        
        DEFINE FIELD IF NOT EXISTS issued_at ON TABLE sessiontoken TYPE Datetime
            DEFAULT time.now()
            PERMISSIONS FOR select, create WHERE true 
            PERMISSIONS FOR update, delete WHERE false;
        
        DEFINE FIELD IF NOT EXISTS expires_at ON TABLE sessiontoken TYPE Datetime
            PERMISSIONS FOR select, create WHERE true 
            PERMISSIONS FOR update, delete WHERE false;
        
        DEFINE FIELD IF NOT EXISTS revoked ON TABLE sessiontoken TYPE bool DEFAULT false;
        
        DEFINE INDEX IF NOT EXISTS jtiIndex ON TABLE sessiontoken FIELDS jti UNIQUE;"#;

        let resp = db.query(query_str).await;

        match resp {
            Ok(_) => {
                println!("SessionToken Table Initialized...");
                Ok(())
            },
            Err(e) => {
                // dbg!(e);
                println!("SessionTokens DB Error : {:?}",e);

                Err(e)
            }
        }
    }

    async fn get_all(db: &Surreal<Client>) -> Vec<Self::Model> {
        let query = db.query("SELECT * FROM sessiontoken").await;
        let mut response = query.ok().unwrap();
        let all_session_tokens = response.take(0).
            ok().unwrap();
        all_session_tokens
    }

    async fn save(self, db: &Surreal<Client>) -> Result<Self::Model, Error> {
        match self.id.clone() {
            None => {
                let session_token: Option<SessionToken> = db.create("sessiontoken").content(self).await?;
                session_token.ok_or(Api(Query("Failed to create session_token".to_string())))
            }
            Some(id) => {
                let session_token:Option<SessionToken> = db.update(id).content(self).await?;
                session_token.ok_or(Api(Query("Failed to update session_token".to_string())))
            }
        }
    }
}

impl SessionToken {
    pub fn new(jti: Uuid, user: &User, issued_at : usize, expires_at : usize) -> Self {
        let userid = user.id.clone().unwrap();
        SessionToken {
            id : None,
            jti,
            user : userid,
            issued_at : Datetime::from(Utc.timestamp_opt(issued_at as i64, 0)
                .single()
                .expect("Invalid timestamp")),
            expires_at : Datetime::from(Utc.timestamp_opt(expires_at as i64, 0)
                .single()
                .expect("Invalid timestamp")),
            revoked: false
        }
    }
}
