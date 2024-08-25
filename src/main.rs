use actix_cors::Cors;
use actix_web::{web, App, HttpServer, Responder};

mod config;
mod controller;
mod service;

static HLS_INDEX_FILE_MEDIA_TYPE: &str = "application/vnd.apple.mpegurl";
static HLS_PART_FILE_MEDIA_TYPE: &str = "video/mp2t";

async fn healthcheck() -> impl Responder {
    "OK"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let minio = config::minio::Minio::new();
    minio.resolve_buckets().await;

    // let track_service = TrackService::new(&minio);

    let state = web::Data::new(config::state::AppState { minio });

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(state.clone())
            .service(controller::track::upload)
            .service(controller::track::get_playlist)
            .service(controller::track::get_part)
            .route("/health", web::get().to(healthcheck))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
