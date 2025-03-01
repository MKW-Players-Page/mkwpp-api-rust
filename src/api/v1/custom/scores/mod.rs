use actix_web::{dev::HttpServiceFactory, web};

mod chart;
mod recent;
mod records;
mod timesheet;

pub fn scores() -> impl HttpServiceFactory {
    return web::scope("/scores")
        .service(recent::recent())
        .service(chart::chart())
        .service(records::records())
        .service(timesheet::timesheet())
        .default_service(web::get().to(default));
}
default_paths_fn!(
    "/recent",
    "/chart/:trackId",
    "/timesheet/:playerId",
    "/records"
);
