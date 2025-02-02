pub mod database;
pub mod routes;
pub mod utils;

#[cfg(test)]
mod tests {
    use argon2::{
        password_hash::{
            rand_core::OsRng,
            PasswordHash, PasswordHasher, PasswordVerifier, SaltString
        },
        Argon2
    };
    use jsonwebtoken::{DecodingKey, EncodingKey};
    use ring::signature::{Ed25519KeyPair, KeyPair};

    #[test]
    fn pasword_hash() -> Result<(), Box<dyn std::error::Error>> {

        let password = "hunter4"; // Bad password; don't actually use!
        let salt = SaltString::generate(&mut OsRng);

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();

        let parsed_hash = PasswordHash::new(&password_hash)?;
        assert!(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok());
        Ok(())
    }


    #[test]
    fn test_ed_dsa_keygen() {
        let doc = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let encoding_key = EncodingKey::from_ed_der(doc.as_ref());

        let pair = Ed25519KeyPair::from_pkcs8(doc.as_ref()).unwrap();
        println!("{:?}", doc.as_ref().to_vec());
        let decoding_key = DecodingKey::from_ed_der(pair.public_key().as_ref());
    }
    
}