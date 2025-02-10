use actix_web::{dev::HttpServiceFactory, web};

mod raw;

pub fn v1() -> impl HttpServiceFactory {
    return web::scope("/v1").service(
        web::scope("/raw")
            .default_service(web::get().to(default))
            .route(
                "/player_awards",
                web::get().to(raw::get::<crate::sql::tables::awards::Awards>),
            )
            .route(
                "/site_champs",
                web::get().to(raw::get::<crate::sql::tables::champs::Champs>),
            )
            .route("/cups", web::get().to(raw::cups::get))
            .route(
                "/edit_submissions",
                web::get().to(raw::get::<crate::sql::tables::edit_submissions::EditSubmissions>),
            )
            .route(
                "/players",
                web::get().to(raw::get::<crate::sql::tables::players::Players>),
            )
            .route(
                "/regions",
                web::get().to(raw::get::<crate::sql::tables::regions::Regions>),
            )
            .route(
                "/scores",
                web::get().to(raw::get::<crate::sql::tables::scores::Scores>),
            )
            .route(
                "/standard_levels",
                web::get().to(raw::get::<crate::sql::tables::standard_levels::StandardLevels>),
            )
            .route(
                "/standards",
                web::get().to(raw::get::<crate::sql::tables::standards::Standards>),
            )
            .route(
                "/submissions",
                web::get().to(raw::get::<crate::sql::tables::submissions::Submissions>),
            )
            .route(
                "/tracks",
                web::get().to(raw::get::<crate::sql::tables::tracks::Tracks>),
            ),
    );
}

async fn default() -> impl actix_web::Responder {
    return actix_web::HttpResponse::Ok().content_type("application/json").body(r#"{"paths":["/player_awards","/site_champs","/cups","/edit_submissions","/players","/regions","/scores","/standard_levels","/standards","/submissions","/tracks"]}"#);
}

pub fn generate_error_json_string(personal_err: &str, lib_err: &str) -> String {
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
    return escaped;
}
