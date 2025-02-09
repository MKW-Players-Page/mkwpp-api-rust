use actix_web::{dev::HttpServiceFactory, web};

mod cups;
mod tracks;

pub fn v1() -> impl HttpServiceFactory {
    return web::scope("/v1").service(
        web::scope("/raw")
            .route("/player_awards", web::get().to(cups::get))
            .route("/site_champs", web::get().to(cups::get))
            .route("/cups", web::get().to(cups::get))
            .route("/edit_submissions", web::get().to(cups::get))
            .route("/players", web::get().to(cups::get))
            .route("/regions", web::get().to(cups::get))
            .route("/scores", web::get().to(cups::get))
            .route("/standard_levels", web::get().to(cups::get))
            .route("/standards", web::get().to(cups::get))
            .route("/submissions", web::get().to(cups::get))
            .route("/tracks", web::get().to(tracks::get)),
    );
}

fn generate_error_json_string(personal_err: &str, lib_err: &str) -> String {
    return format!(
        "{{\"error\":\"{}\",\"server_error\":\"{}\"}}",
        escape_char_for_json(personal_err),
        escape_char_for_json(lib_err)
    );
}

fn escape_char_for_json(src: &str) -> String {
    use std::fmt::Write;
    let mut escaped = String::with_capacity(src.len());
    let mut utf16_buf = [0u16; 2];
    for c in src.chars() {
        match c {
            '\x08' => escaped += "\\b",
            '\x0c' => escaped += "\\f",
            '\n' => escaped += "\\n",
            '\r' => escaped += "\\r",
            '\t' => escaped += "\\t",
            '"' => escaped += "\\\"",
            '\\' => escaped += "\\",
            c if c.is_ascii_graphic() => escaped.push(c),
            c => {
                let encoded = c.encode_utf16(&mut utf16_buf);
                for utf16 in encoded {
                    write!(&mut escaped, "\\u{:04X}", utf16).unwrap();
                }
            }
        }
    }
    escaped
}
