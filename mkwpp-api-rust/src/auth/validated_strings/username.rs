use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

use super::ValidatedString;

#[derive(sqlx::FromRow)]
pub struct Username(String);

impl ValidatedString for Username {
    fn new_from_string(val: String) -> Result<Self, FinalErrorResponse> {
        match val.len() {
            0..=4 => Err(EveryReturnedError::UsernameTooShort.into_final_error("")),
            151.. => Err(EveryReturnedError::UsernameTooLong.into_final_error("")),
            _ => Ok(Self(val)),
        }
    }

    fn get_inner(self) -> String {
        self.0
    }
}
