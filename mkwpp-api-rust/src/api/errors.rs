use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError, http::StatusCode};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(serde::Serialize, Debug)]
pub struct FinalErrorResponse {
    #[serde(skip)]
    status_code: StatusCode,

    error_code: u64,
    non_field_errors: Vec<String>,
    field_errors: std::collections::HashMap<String, Vec<String>>,
}

impl FinalErrorResponse {
    fn new(
        error_code: u64,
        status_code: StatusCode,
        non_field_errors: Vec<String>,
        field_errors: std::collections::HashMap<String, Vec<String>>,
    ) -> Self {
        FinalErrorResponse {
            status_code,
            error_code,
            non_field_errors,
            field_errors,
        }
    }

    fn generate_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code).json(self)
    }
}

impl Display for FinalErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:#?}")
    }
}

impl ResponseError for FinalErrorResponse {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        self.generate_response()
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        self.status_code
    }
}

pub enum EveryReturnedError {
    NoConnectionFromPGPool,
    SerializingDataToJSON,
    ClosingConnectionFromPGPool,
    GettingFromDatabase,
    DecodingDatabaseRows,
    _UserIdToPlayerId,
    _GenerateTimesheet,
    _GenerateMatchup,
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
    _GeneratingToken,
    MismatchedIds,
    NothingChanged,
    InvalidInput,
    TechnicallyUnreachableCode,
    CreatingEmailClient,
    SendingEmail,
    UserNotVerified,
    UserOnCooldown,
}

impl From<EveryReturnedError> for u64 {
    fn from(val: EveryReturnedError) -> Self {
        match val {
            EveryReturnedError::NoConnectionFromPGPool => 0,
            EveryReturnedError::SerializingDataToJSON => 1,
            EveryReturnedError::ClosingConnectionFromPGPool => 2,
            EveryReturnedError::GettingFromDatabase => 3,
            EveryReturnedError::DecodingDatabaseRows => 4,
            EveryReturnedError::_UserIdToPlayerId => 5,
            EveryReturnedError::_GenerateTimesheet => 6,
            EveryReturnedError::_GenerateMatchup => 7,
            EveryReturnedError::UsernameTooShort => 8,
            EveryReturnedError::UsernameTooLong => 9,
            EveryReturnedError::PasswordTooLong => 10,
            EveryReturnedError::PasswordTooShort => 11,
            EveryReturnedError::PasswordMustHaveSpecial => 12,
            EveryReturnedError::PasswordMustHaveLowercase => 13,
            EveryReturnedError::PasswordMustHaveUppercase => 14,
            EveryReturnedError::PasswordMustHaveNumber => 15,
            EveryReturnedError::EmailTooLong => 16,
            EveryReturnedError::EmailInvalid => 17,
            EveryReturnedError::UserIDDoesntExist => 18,
            EveryReturnedError::InvalidSessionToken => 19,
            EveryReturnedError::UserHasNoAssociatedPlayer => 20,
            EveryReturnedError::CreatePGTransaction => 21,
            EveryReturnedError::CommitPGTransaction => 22,
            EveryReturnedError::InsufficientPermissions => 23,
            EveryReturnedError::_GeneratingToken => 24,
            EveryReturnedError::MismatchedIds => 25,
            EveryReturnedError::NothingChanged => 26,
            EveryReturnedError::InvalidInput => 27,
            EveryReturnedError::TechnicallyUnreachableCode => 28,
            EveryReturnedError::CreatingEmailClient => 29,
            EveryReturnedError::SendingEmail => 30,
            EveryReturnedError::UserNotVerified => 31,
            EveryReturnedError::UserOnCooldown => 32,
        }
    }
}

impl EveryReturnedError {
    pub fn into_final_error(self, library_error: impl ToString) -> FinalErrorResponse {
        let mut out = match self {
            Self::NoConnectionFromPGPool => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("Couldn't get connection from data pool")],
                HashMap::new(),
            ),
            Self::SerializingDataToJSON => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("Error serializing data to JSON")],
                HashMap::new(),
            ),
            Self::ClosingConnectionFromPGPool => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("Error closing Database connection")],
                HashMap::new(),
            ),
            Self::GettingFromDatabase => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("Couldn't get rows from database")],
                HashMap::new(),
            ),
            Self::DecodingDatabaseRows => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("Error decoding database rows")],
                HashMap::new(),
            ),
            Self::_UserIdToPlayerId => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("Error converting User ID to Player ID")],
                HashMap::new(),
            ),
            Self::_GenerateTimesheet => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("Error generating timesheet")],
                HashMap::new(),
            ),
            Self::_GenerateMatchup => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("Error generating matchup")],
                HashMap::new(),
            ),

            Self::UsernameTooShort => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Error validating the username")],
                std::collections::HashMap::from([(
                    String::from("username"),
                    vec![String::from("Username too short")],
                )]),
            ),
            Self::UsernameTooLong => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Error validating the username")],
                std::collections::HashMap::from([(
                    String::from("username"),
                    vec![String::from("Username too long")],
                )]),
            ),
            Self::PasswordTooLong => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Error validating the password")],
                std::collections::HashMap::from([(
                    String::from("password"),
                    vec![String::from("Password too long")],
                )]),
            ),
            Self::PasswordTooShort => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Error validating the password")],
                std::collections::HashMap::from([(
                    String::from("password"),
                    vec![String::from("Password too short")],
                )]),
            ),
            Self::PasswordMustHaveSpecial => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Error validating the password")],
                std::collections::HashMap::from([(
                    String::from("password"),
                    vec![String::from("Password must have a special character")],
                )]),
            ),
            Self::PasswordMustHaveLowercase => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Error validating the password")],
                std::collections::HashMap::from([(
                    String::from("password"),
                    vec![String::from("Password must have a lowercase character")],
                )]),
            ),
            Self::PasswordMustHaveUppercase => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Error validating the password")],
                std::collections::HashMap::from([(
                    String::from("password"),
                    vec![String::from("Password must have an uppercase character")],
                )]),
            ),
            Self::PasswordMustHaveNumber => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Error validating the password")],
                std::collections::HashMap::from([(
                    String::from("password"),
                    vec![String::from("Password must have a number")],
                )]),
            ),
            Self::EmailTooLong => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Error validating the email")],
                std::collections::HashMap::from([(
                    String::from("email"),
                    vec![String::from("Email too long")],
                )]),
            ),
            Self::EmailInvalid => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Error validating the email")],
                std::collections::HashMap::from([(
                    String::from("email"),
                    vec![String::from("Email invalid")],
                )]),
            ),
            Self::UserIDDoesntExist => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("Error getting user ID")],
                HashMap::new(),
            ),

            Self::InvalidSessionToken => FinalErrorResponse::new(
                self.into(),
                StatusCode::FORBIDDEN,
                vec![String::from("Invalid session token")],
                HashMap::new(),
            ),
            Self::UserHasNoAssociatedPlayer => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("User has no associated player")],
                HashMap::new(),
            ),
            Self::CreatePGTransaction => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("Error creating postgres transaction")],
                HashMap::new(),
            ),
            Self::CommitPGTransaction => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("Error committing postgres transaction")],
                HashMap::new(),
            ),
            Self::InsufficientPermissions => FinalErrorResponse::new(
                self.into(),
                StatusCode::FORBIDDEN,
                vec![String::from("Insufficient Permissions")],
                HashMap::new(),
            ),
            Self::_GeneratingToken => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("Error generating token")],
                HashMap::new(),
            ),
            Self::MismatchedIds => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Mismatched IDs")],
                HashMap::new(),
            ),
            Self::NothingChanged => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Nothing to update")],
                HashMap::new(),
            ),
            Self::InvalidInput => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("Input is invalid")],
                HashMap::new(),
            ),
            Self::TechnicallyUnreachableCode => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from(
                    "Technically unreachable code has been reached",
                )],
                HashMap::new(),
            ),
            Self::CreatingEmailClient => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("There was an error creating the email client")],
                HashMap::new(),
            ),
            Self::SendingEmail => FinalErrorResponse::new(
                self.into(),
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![String::from("There was an error sending the email")],
                HashMap::new(),
            ),
            Self::UserNotVerified => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("User is not verified")],
                HashMap::new(),
            ),
            Self::UserOnCooldown => FinalErrorResponse::new(
                self.into(),
                StatusCode::BAD_REQUEST,
                vec![String::from("User is on cooldown")],
                HashMap::new(),
            ),
        };

        let library_error = library_error.to_string();
        if !library_error.is_empty() {
            out.non_field_errors.push(library_error);
        }

        out
    }
}
