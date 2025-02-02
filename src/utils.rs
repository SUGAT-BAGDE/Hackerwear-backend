use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
pub struct AppState {
    pub db: Surreal<Client>,
    pub jwt_key_pair : auth::JwtKeyPair
}

impl AppState {
    pub fn new(db: Surreal<Client>) -> AppState {
        AppState{
            db,
            jwt_key_pair : init_jwt_keys()
        }
    }
}


fn init_jwt_keys() -> auth::JwtKeyPair {
    auth::generate_ed_dsa_keypair()
}

pub mod auth {
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
    struct Claims {
        iss: String, // issuer
        sub: String, // user
        aud: String, // audience
        exp: usize,  // expiry
        iat: usize,  // issued at
        jti: Uuid    // uuid of token
    }

    pub async fn generate_jwt(user: &super::super::database::models::User, keypair : &JwtKeyPair, db: &Surreal<Client>) -> String {
        let iat = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let exp = iat + 10 * 24 * 60 * 60;
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

    pub fn validate_jwt(token : &str, keypair : JwtKeyPair) -> bool {
        let validation = Validation::new(Algorithm::EdDSA);
        match decode::<Claims>(token, &(keypair.decoding_key), &validation) { 
            Ok(_) => true,
            Err(_) => false
        }
    }

    pub fn generate_ed_dsa_keypair() -> JwtKeyPair {
        let doc = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let encoding_key = EncodingKey::from_ed_der(doc.as_ref());

        let pair = Ed25519KeyPair::from_pkcs8(doc.as_ref()).unwrap();
        let decoding_key = DecodingKey::from_ed_der(pair.public_key().as_ref());
        JwtKeyPair { encoding_key, decoding_key }
    }
}