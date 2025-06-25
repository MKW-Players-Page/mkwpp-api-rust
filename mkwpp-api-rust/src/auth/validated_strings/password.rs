use base64::Engine;

use crate::api::errors::EveryReturnedError;

use super::ValidatedString;

#[derive(sqlx::FromRow, Clone)]
pub struct Password(String);

impl ValidatedString for Password {
    type Err = EveryReturnedError;

    fn new_from_string(val: String) -> Result<Self, Self::Err> {
        let val = match val.len() {
            0..=8 => return Err(EveryReturnedError::PasswordTooShort),
            129.. => return Err(EveryReturnedError::PasswordTooLong),
            _ => val,
        };

        let mut has_uppercase = false;
        let mut has_lowercase = false;
        let mut has_special_character = false;
        let mut has_number = false;
        for character in val.chars() {
            if !character.is_alphanumeric() {
                has_special_character = true;
                continue;
            }

            if character.is_numeric() {
                has_number = true;
                continue;
            }

            if character.is_lowercase() {
                has_lowercase = true;
                continue;
            }

            if character.is_uppercase() {
                has_uppercase = true;
                continue;
            }
        }

        if !has_uppercase {
            return Err(EveryReturnedError::PasswordMustHaveUppercase);
        }
        if !has_lowercase {
            return Err(EveryReturnedError::PasswordMustHaveLowercase);
        }
        if !has_special_character {
            return Err(EveryReturnedError::PasswordMustHaveSpecial);
        }
        if !has_number {
            return Err(EveryReturnedError::PasswordMustHaveNumber);
        }

        Ok(Self(val))
    }

    fn get_inner(self) -> String {
        self.0
    }
}

impl Password {
    pub fn hash(self, salt: &[u8]) -> String {
        let argon = argon2::Argon2::default();

        let mut hash_bytes = [0u8; 189];
        argon
            .hash_password_into(self.get_inner().as_bytes(), salt, &mut hash_bytes)
            .expect("Password failed to hash, this should be infallible");

        let token_engine = base64::engine::GeneralPurpose::new(
            &base64::alphabet::URL_SAFE,
            base64::engine::GeneralPurposeConfig::new(),
        );

        let mut out_string = String::new();
        token_engine.encode_string(hash_bytes, &mut out_string);
        out_string
    }
}
