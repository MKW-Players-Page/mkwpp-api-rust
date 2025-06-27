use actix_web::{HttpResponse, HttpResponseBuilder, body::MessageBody};

pub mod errors;
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
