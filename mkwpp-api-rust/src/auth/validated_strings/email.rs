use super::ValidatedString;

#[derive(Debug)]
pub enum EmailError {
    TooLong,
    Invalid,
}

#[derive(sqlx::FromRow)]
pub struct Email(String);

impl ValidatedString for Email {
    type Err = EmailError;
    fn new_from_string(val: String) -> Result<Self, Self::Err> {
        if val.len() > 254 {
            return Err(EmailError::TooLong);
        }

        let regex_checker = regex::Regex::new(r"^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
        match regex_checker.is_match(&val) {
            true => Ok(Self(val)),
            false => Err(EmailError::Invalid),
        }
    }

    fn get_inner(self) -> String {
        self.0
    }
}
