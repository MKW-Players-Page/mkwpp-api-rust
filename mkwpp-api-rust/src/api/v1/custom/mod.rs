use actix_web::{dev::HttpServiceFactory, web};

mod blog;
pub mod params;
mod players;
mod rankings;
mod regions;
mod scores;
mod site_champs;

pub fn custom() -> impl HttpServiceFactory {
    web::scope("/custom")
        .service(scores::scores())
        .service(rankings::rankings())
        .service(regions::regions())
        .service(players::players())
        .service(blog::blog())
        .service(site_champs::site_champs())
        .default_service(web::get().to(default))
}
default_paths_fn!(
    "/scores",
    "/rankings",
    "/regions",
    "/players",
    "/site_champs"
);
