use actix_web::{dev::HttpServiceFactory, web};

pub mod params;
mod players;
mod rankings;
mod regions;
mod scores;

pub fn custom() -> impl HttpServiceFactory {
    return web::scope("/custom")
        .service(scores::scores())
        .service(rankings::rankings())
        .service(regions::regions())
        .service(players::players())
        .default_service(web::get().to(default));
}
default_paths_fn!("/scores", "/rankings", "/regions", "/players");
