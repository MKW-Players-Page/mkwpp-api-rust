use actix_web::{HttpResponse, HttpResponseBuilder, body::MessageBody};

pub mod v1;

pub fn read_file(
    file_path: &str,
    mime_type: &str,
    builder: &mut HttpResponseBuilder,
) -> HttpResponse {
    builder.content_type(mime_type);
    response_builder(
        builder,
        std::fs::read_to_string(file_path).expect("Missing file"),
    )
}

pub fn response_builder(
    builder: &mut HttpResponseBuilder,
    body: impl MessageBody + 'static,
) -> HttpResponse {
    builder.body(body)
}

#[derive(serde::Serialize)]
pub struct FinalErrorResponse {
    non_field_errors: Vec<String>,
    field_errors: std::collections::HashMap<String, Vec<String>>,
}

impl FinalErrorResponse {
    pub fn new(
        non_field_errors: Vec<String>,
        field_errors: std::collections::HashMap<String, Vec<String>>,
    ) -> Self {
        FinalErrorResponse {
            non_field_errors,
            field_errors,
        }
    }
    pub fn new_no_fields(non_field_errors: Vec<String>) -> Self {
        FinalErrorResponse::new(non_field_errors, std::collections::HashMap::new())
    }

    pub fn generate_response(
        &self,
        callback: impl FnOnce() -> HttpResponseBuilder,
    ) -> HttpResponse {
        let x = serde_json::to_string(self).unwrap();
        callback().content_type("application/json").body(x)
    }
}
