pub mod email;
pub mod password;
pub mod username;

pub trait ValidatedString<'a> {
    type Err;

    fn new_from_string(val: String) -> Result<Self, Self::Err>
    where
        Self: Sized;

    fn get_inner(self) -> String;
}
