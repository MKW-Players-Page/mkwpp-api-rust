use crate::api::errors::FinalErrorResponse;

pub mod email;
pub mod password;
pub mod username;

pub trait ValidatedString {
    fn new_from_string(val: String) -> Result<Self, FinalErrorResponse>
    where
        Self: Sized;

    fn get_inner(self) -> String;
}
