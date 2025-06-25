use std::collections::HashMap;

use actix_web::{HttpResponse, HttpResponseBuilder};

#[derive(serde::Serialize)]
struct FinalErrorResponse {
    error_code: u64,
    non_field_errors: Vec<String>,
    field_errors: std::collections::HashMap<String, Vec<String>>,
}

impl FinalErrorResponse {
    fn new(
        error_code: u64,
        non_field_errors: Vec<String>,
        field_errors: std::collections::HashMap<String, Vec<String>>,
    ) -> Self {
        FinalErrorResponse {
            error_code,
            non_field_errors,
            field_errors,
        }
    }

    fn generate_response(&self, callback: impl FnOnce() -> HttpResponseBuilder) -> HttpResponse {
        let x = serde_json::to_string(self).unwrap();
        callback().content_type("application/json").body(x)
    }
}

pub enum EveryReturnedError {
    NoConnectionFromPGPool,
    SerializingDataToJSON,
    ClosingConnectionFromPGPool,
    GettingFromDatabase,
    DecodingDatabaseRows,
    UserIdToPlayerId,
    GenerateTimesheet,
    GenerateMatchup,
    UsernameTooShort,
    UsernameTooLong,
    PasswordTooLong,
    PasswordTooShort,
    PasswordMustHaveSpecial,
    PasswordMustHaveLowercase,
    PasswordMustHaveUppercase,
    PasswordMustHaveNumber,
    EmailTooLong,
    EmailInvalid,
    UserIDDoesntExist,
    InvalidSessionToken,
    UserHasNoAssociatedPlayer,
    CreatePGTransaction,
    CommitPGTransaction,
    InsufficientPermissions,
    GeneratingToken,
    MismatchedIds,
    NothingChanged,
}

impl Into<u64> for EveryReturnedError {
    fn into(self) -> u64 {
        match self {
            Self::NoConnectionFromPGPool => 0,
            Self::SerializingDataToJSON => 1,
            Self::ClosingConnectionFromPGPool => 2,
            Self::GettingFromDatabase => 3,
            Self::DecodingDatabaseRows => 4,
            Self::UserIdToPlayerId => 5,
            Self::GenerateTimesheet => 6,
            Self::GenerateMatchup => 7,
            Self::UsernameTooShort => 8,
            Self::UsernameTooLong => 9,
            Self::PasswordTooLong => 10,
            Self::PasswordTooShort => 11,
            Self::PasswordMustHaveSpecial => 12,
            Self::PasswordMustHaveLowercase => 13,
            Self::PasswordMustHaveUppercase => 14,
            Self::PasswordMustHaveNumber => 15,
            Self::EmailTooLong => 16,
            Self::EmailInvalid => 17,
            Self::UserIDDoesntExist => 18,
            Self::InvalidSessionToken => 19,
            Self::UserHasNoAssociatedPlayer => 20,
            Self::CreatePGTransaction => 21,
            Self::CommitPGTransaction => 22,
            Self::InsufficientPermissions => 23,
            Self::GeneratingToken => 24,
            Self::MismatchedIds => 25,
            Self::NothingChanged => 26,
        }
    }
}

impl EveryReturnedError {
    pub fn http_response(self, library_error: impl ToString) -> HttpResponse {
        match self {
            Self::NoConnectionFromPGPool => FinalErrorResponse::new(
                self.into(),
                vec![
                    String::from("Couldn't get connection from data pool"),
                    library_error.to_string(),
                ],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),
            Self::SerializingDataToJSON => FinalErrorResponse::new(
                self.into(),
                vec![
                    String::from("Error serializing data to JSON"),
                    library_error.to_string(),
                ],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),
            Self::ClosingConnectionFromPGPool => FinalErrorResponse::new(
                self.into(),
                vec![
                    String::from("Error closing Database connection"),
                    library_error.to_string(),
                ],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),
            Self::GettingFromDatabase => FinalErrorResponse::new(
                self.into(),
                vec![
                    String::from("Couldn't get rows from database"),
                    library_error.to_string(),
                ],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),
            Self::DecodingDatabaseRows => FinalErrorResponse::new(
                self.into(),
                vec![
                    String::from("Error decoding database rows"),
                    library_error.to_string(),
                ],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),
            Self::UserIdToPlayerId => FinalErrorResponse::new(
                self.into(),
                vec![
                    String::from("Error converting User ID to Player ID"),
                    library_error.to_string(),
                ],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),
            Self::GenerateTimesheet => FinalErrorResponse::new(
                self.into(),
                vec![
                    String::from("Error generating timesheet"),
                    library_error.to_string(),
                ],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),
            Self::GenerateMatchup => FinalErrorResponse::new(
                self.into(),
                vec![
                    String::from("Error generating matchup"),
                    library_error.to_string(),
                ],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),

            Self::UsernameTooShort => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error validating the username")],
                std::collections::HashMap::from([(
                    String::from("username"),
                    vec![String::from("Username too short")],
                )]),
            )
            .generate_response(HttpResponse::BadRequest),
            Self::UsernameTooLong => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error validating the username")],
                std::collections::HashMap::from([(
                    String::from("username"),
                    vec![String::from("Username too long")],
                )]),
            )
            .generate_response(HttpResponse::BadRequest),
            Self::PasswordTooLong => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error validating the password")],
                std::collections::HashMap::from([(
                    String::from("password"),
                    vec![String::from("Password too long")],
                )]),
            )
            .generate_response(HttpResponse::BadRequest),
            Self::PasswordTooShort => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error validating the password")],
                std::collections::HashMap::from([(
                    String::from("password"),
                    vec![String::from("Password too short")],
                )]),
            )
            .generate_response(HttpResponse::BadRequest),
            Self::PasswordMustHaveSpecial => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error validating the password")],
                std::collections::HashMap::from([(
                    String::from("password"),
                    vec![String::from("Password must have a special character")],
                )]),
            )
            .generate_response(HttpResponse::BadRequest),
            Self::PasswordMustHaveLowercase => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error validating the password")],
                std::collections::HashMap::from([(
                    String::from("password"),
                    vec![String::from("Password must have a lowercase character")],
                )]),
            )
            .generate_response(HttpResponse::BadRequest),
            Self::PasswordMustHaveUppercase => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error validating the password")],
                std::collections::HashMap::from([(
                    String::from("password"),
                    vec![String::from("Password must have an uppercase character")],
                )]),
            )
            .generate_response(HttpResponse::BadRequest),
            Self::PasswordMustHaveNumber => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error validating the password")],
                std::collections::HashMap::from([(
                    String::from("password"),
                    vec![String::from("Password must have a number")],
                )]),
            )
            .generate_response(HttpResponse::BadRequest),
            Self::EmailTooLong => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error validating the email")],
                std::collections::HashMap::from([(
                    String::from("email"),
                    vec![String::from("Email too long")],
                )]),
            )
            .generate_response(HttpResponse::BadRequest),
            Self::EmailInvalid => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error validating the email")],
                std::collections::HashMap::from([(
                    String::from("email"),
                    vec![String::from("Email invalid")],
                )]),
            )
            .generate_response(HttpResponse::BadRequest),
            Self::UserIDDoesntExist => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error getting user ID")],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),

            Self::InvalidSessionToken => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Invalid session token")],
                HashMap::new(),
            )
            .generate_response(HttpResponse::Forbidden),
            Self::UserHasNoAssociatedPlayer => FinalErrorResponse::new(
                self.into(),
                vec![String::from("User has no associated player")],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),
            Self::CreatePGTransaction => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error creating postgres transaction")],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),
            Self::CommitPGTransaction => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error committing postgres transaction")],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),
            Self::InsufficientPermissions => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Insufficient Permissions")],
                HashMap::new(),
            )
            .generate_response(HttpResponse::Forbidden),
            Self::GeneratingToken => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Error generating token")],
                HashMap::new(),
            )
            .generate_response(HttpResponse::InternalServerError),
            Self::MismatchedIds => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Mismatched IDs")],
                HashMap::new(),
            )
            .generate_response(HttpResponse::BadRequest),
            Self::NothingChanged => FinalErrorResponse::new(
                self.into(),
                vec![String::from("Nothing to update")],
                HashMap::new(),
            )
            .generate_response(HttpResponse::BadRequest),
        }
    }
}
