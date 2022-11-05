use snafu::{prelude::*, Whatever};

use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
pub enum MyCustomError {
    HttpError,
    ParseError,
}
pub fn hash_password<P: Into<String>, S: Into<String>>(secret: P, salt: S) -> Result< String,Whatever >{
    let password = secret.into(); // Bad password; don't actually use!
    let salt = SaltString::b64_encode(salt.into().as_bytes())?;

    // Hash password to PHC string ($pbkdf2-sha256$...)
    let password_hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    // Verify password against PHC string
    let parsed_hash = PasswordHash::new(&password_hash).unwrap();

    Ok(parsed_hash.hash.unwrap().to_string())
}

#[cfg(test)]
mod tests {
    use crate::hash_password;

    #[test]
    fn it_works() {
     let var =    hash_password("bobss", " fgdfgsalt").unwrap();
        println!("yo {}",var);
        let s: String = String::from("bob");
        hash_password(s, " fgdfgsalt").unwrap();
    }
}
