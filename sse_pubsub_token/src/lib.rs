use snafu::prelude::*;
use std::fmt;
use std::fmt::{Debug, Display};

use crate::SseError::Pbkdf2Error;
use pbkdf2::password_hash::Error;
use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};

pub struct DisplayError<T>(pub T);

impl<T> Debug for DisplayError<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Display for DisplayError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> snafu::Error for DisplayError<T> where T: Display + Debug {}

#[derive(Debug, Snafu)]
pub enum SseError {
    Pbkdf2Error {
        source: DisplayError<pbkdf2::password_hash::Error>,
    },
    ParseError,
}

impl From<Error> for SseError {
    fn from(error: Error) -> Self {
        Pbkdf2Error {
            source: DisplayError(error),
        }
    }
}
/// Password Based Key Derivation Function
/// Derive key data from a secret + salt
pub fn hash_password<P: Into<String>, S: Into<String>>(
    secret: P,
    salt: S,
) -> Result<String, SseError> {
    let password = secret.into();
    let salt = SaltString::b64_encode(salt.into().as_bytes())?;
    // Hash password to PHC string ($pbkdf2-sha256$...)
    let password_hash = Pbkdf2.hash_password(password.as_bytes(), &salt)?;

    Ok(password_hash.hash.unwrap().to_string())
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use snafu::AsErrorSource;
    use crate::{hash_password, SseError};

    #[test]
    fn it_works() {
        let var = hash_password("bobss", " fgdfgsalt").unwrap();
        println!("yo {}", var);
        let s: String = String::from("bob");
        if let Ok(something) =   hash_password(&s, "x".repeat(100))  {
            println!("was ok {}",something);
        }
        else {
            println!("yo error");
        }
        match hash_password(s, "x".repeat(100))  {
            Ok(value) => {println!("{}",value);}
            Err(err) => {
                match err  {
                    SseError::Pbkdf2Error { source } => {

                        println!("{:?}",source.0)}
                    SseError::ParseError => {}
                }
            }
        }
    }
}
