use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::{Error, Surreal};

pub trait DatabaseIO {
    type Model : Serialize + Deserialize<'static>;
    
    async fn init(db : &Surreal<Client>) -> Result<(), Error>;
    async fn get_all(db : &Surreal<Client>) -> Vec<Self::Model>;
    async fn save(self, db : &Surreal<Client>) -> Result<Self::Model, Error>;
}

pub mod password_utils
{
    use argon2::{
        password_hash::{
            rand_core::OsRng,
            PasswordHash, PasswordHasher, PasswordVerifier, SaltString
        },
        Argon2,
        password_hash::Error as PasswordHashError
    };
    
    pub fn hash_password(password :  &str) -> Result<String, PasswordHashError> {
        
        let salt = SaltString::generate(&mut OsRng);
    
        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();
    
        // Hash password to PHC string ($argon2id$v=19$...)
        Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
        
    }
    
    pub fn verify_password(hash : &str, password : &str) -> Result<bool, PasswordHashError> {
        let parsed_hash = PasswordHash::new(hash)?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}