use super::ValidatedString;

#[derive(Debug)]
pub enum UsernameError {
    TooLong,
    TooShort,
    // Invalid,
}

#[derive(sqlx::FromRow)]
pub struct Username(String);

impl ValidatedString for Username {
    type Err = UsernameError;

    fn new_from_string(val: String) -> Result<Self, Self::Err> {
        match val.len() {
            0..=4 => Err(UsernameError::TooShort),
            151.. => Err(UsernameError::TooLong),
            _ => Ok(Self(val)),
        }
    }

    fn get_inner(self) -> String {
        self.0
    }
}
