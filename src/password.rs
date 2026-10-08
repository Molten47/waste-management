use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    match PasswordHash::new(stored_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_password_verifies() {
        let hash = hash_password("s3cret-pass").unwrap();
        assert!(verify_password("s3cret-pass", &hash));
    }

    #[test]
    fn wrong_password_fails() {
        let hash = hash_password("s3cret-pass").unwrap();
        assert!(!verify_password("not-the-password", &hash));
    }

    #[test]
    fn same_password_hashes_differently() {
        let a = hash_password("s3cret-pass").unwrap();
        let b = hash_password("s3cret-pass").unwrap();
        assert_ne!(a, b, "each hash must use a fresh random salt");
    }

    #[test]
    fn garbage_hash_is_rejected_not_panicked() {
        assert!(!verify_password("anything", "not-a-real-hash"));
    }
}
