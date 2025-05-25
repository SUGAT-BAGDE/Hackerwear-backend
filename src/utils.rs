use std::path::Path;

use auth::{generate_ed_dsa_keypair, get_pkcs8_der};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use std::env;

use crate::database::db::Credentials;

pub struct AppState {
    pub db: Surreal<Client>,
    pub jwt_key_pair : auth::JwtKeyPair
}

impl AppState {
    pub fn new(db: Surreal<Client>, jwt_key_path: &str) -> AppState {
        AppState{
            db,
            jwt_key_pair : init_jwt_keys(jwt_key_path)
        }
    }
}

fn init_jwt_keys(jwt_key_path: &str) -> auth::JwtKeyPair {
    let key_path = Path::new(jwt_key_path);
    let pkcs8_der = get_pkcs8_der(key_path);
    generate_ed_dsa_keypair(&pkcs8_der)
}

pub struct AppConfig{
    pub surreal_hostname : String,
    pub credentials : Credentials,
    pub jwt_key_path : String
}

pub fn extract_app_config_from_env() -> Result<AppConfig, String> {
    let hostname = env::var("SURREAL_HOSTNAME").map_err(|_| "Missing SURREAL_HOSTNAME")?;
    let namespace = env::var("SURREAL_NAMESPACE").map_err(|_| "Missing SURREAL_NAMESPACE")?;
    let username = env::var("SURREAL_USERNAME").map_err(|_| "Missing SURREAL_USERNAME")?;
    let password = env::var("SURREAL_PASSWORD").map_err(|_| "Missing SURREAL_PASSWORD")?;
    let database = env::var("SURREAL_DATABASE").map_err(|_| "Missing SURREAL_DATABASE")?;

    let cred = Credentials{
        namespace,
        username,
        password,
        database
    };

    let jwt_key_path = match env::var("JWT_KEY_PATH") {
        Ok(val) => val,
        Err(_) => {
            #[cfg(debug_assertions)]
            {
                "./security/jwt_private_key.der".to_string()
            }
            #[cfg(not(debug_assertions))]
            {
                "/etc/myapp/jwt_private_key.der".to_string()
            }
        }
    };


    Ok(AppConfig { surreal_hostname: hostname, credentials: cred, jwt_key_path })
}


pub mod auth {
    use std::fs;
    use std::path::Path;
    use std::time::{ SystemTime, UNIX_EPOCH };
    use ring::signature::{ Ed25519KeyPair, KeyPair };
    use jsonwebtoken::{ decode, encode, Algorithm, DecodingKey, EncodingKey, Validation };
    use serde::{ Deserialize, Serialize };
    use surrealdb::engine::remote::ws::Client;
    use surrealdb::Surreal;
    use uuid::Uuid;
    
    use crate::database::models::DatabaseIO;
    use crate::database::models::session_token::SessionToken;

    pub struct JwtKeyPair {
        pub encoding_key: EncodingKey,
        pub decoding_key: DecodingKey
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims {
        iss: String, // issuer
        sub: String, // user
        aud: String, // audience
        exp: usize,  // expiry
        iat: usize,  // issued at
        // Make this private
        pub jti: Uuid    // uuid of token
    }

    pub enum JwtStatus{
        Valid(Claims),
        Expired,
        Invalid
    }

    pub async fn generate_jwt(user: &super::super::database::models::User, keypair : &JwtKeyPair, db: &Surreal<Client>) -> String {
        let iat = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let exp = iat + 10 * 24 * 60 * 60; // 10 days of validity
        let jti = Uuid::new_v4();

        let claims = Claims {
            iss : "hackerwear-api-server".to_string(),
            sub : user.email.clone(),
            aud : "hackerwear-web".to_string(),
            iat,
            exp,
            jti : jti.clone(),
        };

        let session_token = SessionToken::new(surrealdb::sql::Uuid::from(jti), user, iat, exp);
        session_token.save(db).await
            .expect("Unable to save session token");

        let token = encode(&jsonwebtoken::Header::new(Algorithm::EdDSA), &claims, &(keypair.encoding_key))
            .unwrap();

        token
    }

    pub fn validate_jwt(token : &str, keypair : &JwtKeyPair) -> JwtStatus {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_audience(&["hackerwear-web"]);
        match decode::<Claims>(token, &(keypair.decoding_key), &validation) { 
            Ok(data) => JwtStatus::Valid(data.claims),

            Err(e) => {
                // Remove the line just next to comment for testing purpose only
                #[cfg(debug_assertions)]
                println!("Error in JWT : {} {}", e,token);
            
                if e.kind() == &jsonwebtoken::errors::ErrorKind::ExpiredSignature {
                    JwtStatus::Expired
                } else {
                    JwtStatus::Invalid
                }

            }
        }
    }

    pub fn get_pkcs8_der(key_path: &Path) -> Vec<u8> {

        if key_path.exists() {
            // Load existing key
            let der_bytes = fs::read(key_path)
                .expect("Failed to read existing JWT private key file");

            der_bytes
        }
        else {
            let rng = ring::rand::SystemRandom::new();
            let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng)
                .expect("Failed to generate Ed25519 keypair");

            if let Some(parent) = key_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).expect("Failed to create directory for key");
                }
            }

            // Save to disk
            fs::write(key_path, pkcs8.as_ref())
                .expect("Failed to write JWT private key to file");

            pkcs8.as_ref().to_vec()
        }
    }

    pub fn generate_ed_dsa_keypair(der_bytes: &[u8]) -> JwtKeyPair {
        let encoding_key = EncodingKey::from_ed_der(der_bytes);

        let pair = Ed25519KeyPair::from_pkcs8(der_bytes).unwrap();
        let decoding_key = DecodingKey::from_ed_der(pair.public_key().as_ref());
        JwtKeyPair { encoding_key, decoding_key }
    }
}