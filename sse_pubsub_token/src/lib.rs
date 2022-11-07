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
        match error {
            // Error::Algorithm => { Pbkdf2Error { source: error } }
            // Error::B64Encoding(_) => {Pbkdf2Error}
            // Error::Crypto => {Pbkdf2Error}
            // Error::OutputTooShort => {Pbkdf2Error}
            // Error::OutputTooLong => {Pbkdf2Error}
            // Error::ParamNameDuplicated => {Pbkdf2Error}
            // Error::ParamNameInvalid => {Pbkdf2Error}
            // Error::ParamValueInvalid(_) => {Pbkdf2Error}
            // Error::ParamsMaxExceeded => {Pbkdf2Error}
            // Error::Password => {Pbkdf2Error}
            // Error::PhcStringInvalid => {Pbkdf2Error}
            // Error::PhcStringTooShort => {Pbkdf2Error}
            // Error::PhcStringTooLong => {Pbkdf2Error}
            // Error::SaltInvalid(_) => {Pbkdf2Error}
            // Error::Version => {Pbkdf2Error}
            _ => Pbkdf2Error {
                source: DisplayError(error),
            },
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
    use crate::hash_password;

    #[test]
    fn it_works() {
        let var = hash_password("bobss", " fgdfgsalt").unwrap();
        println!("yo {}", var);
        let s: String = String::from("bob");
        if let Ok(something) =   hash_password(s, "x".repeat(100))  {
            println!("was ok {}",something);
        }
        else {
            println!("yo error");
        }
    }
}
