use crate::api::errors::{EveryReturnedError, FinalErrorResponse};

use super::ValidatedString;

#[derive(sqlx::FromRow)]
pub struct Email(String);

impl ValidatedString for Email {
    fn new_from_string(val: String) -> Result<Self, FinalErrorResponse> {
        if val.len() > 254 {
            return Err(EveryReturnedError::EmailTooLong.to_final_error(""));
        }

        let regex_checker = regex::Regex::new(r"^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
        match regex_checker.is_match(&val) {
            true => Ok(Self(val)),
            false => Err(EveryReturnedError::EmailInvalid.to_final_error("")),
        }
    }

    fn get_inner(self) -> String {
        self.0
    }
}
