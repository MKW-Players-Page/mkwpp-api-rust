use crate::api::errors::EveryReturnedError;

use super::ValidatedString;

#[derive(sqlx::FromRow)]
pub struct Username(String);

impl ValidatedString for Username {
    type Err = EveryReturnedError;

    fn new_from_string(val: String) -> Result<Self, Self::Err> {
        match val.len() {
            0..=4 => Err(EveryReturnedError::UsernameTooShort),
            151.. => Err(EveryReturnedError::UsernameTooLong),
            _ => Ok(Self(val)),
        }
    }

    fn get_inner(self) -> String {
        self.0
    }
}
