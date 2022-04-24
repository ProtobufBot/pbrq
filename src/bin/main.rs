#![feature(async_closure)]

use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use pbbot_rq::handler::{bot, password, plugins, qrcode};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_timer(tracing_subscriber::fmt::time::OffsetTime::new(
                    time::UtcOffset::__from_hms_unchecked(8, 0, 0),
                    time::macros::format_description!(
                        "[year]-[month]-[day] [hour]:[minute]:[second]"
                    ),
                )),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("main", Level::DEBUG)
                .with_target("rq_client", Level::DEBUG)
                .with_target("rs_qq", Level::DEBUG),
        )
        .init();
    let app = Router::new()
        .route("/ping", get(async move || "pong"))
        .nest(
            "/login",
            Router::new()
                .nest(
                    "/qrcode",
                    Router::new()
                        .route("/create", post(qrcode::create))
                        .route("/list", get(qrcode::list))
                        .route("/delete", post(qrcode::delete))
                        .route("/query", post(qrcode::query)),
                )
                .nest(
                    "/password",
                    Router::new()
                        .route("/create", post(password::login))
                        .route("/submit_sms", post(password::submit_sms))
                        .route("/submit_ticket", post(password::submit_ticket))
                        .route("/list", get(password::list))
                        .route("/delete", post(password::delete)),
                ),
        )
        .nest(
            "/bot",
            Router::new()
                .route("/list", get(bot::list))
                .route("/delete", post(bot::delete)),
        )
        .nest(
            "/plugin",
            Router::new()
                .route("/save", post(plugins::save))
                .route("/list", get(plugins::list))
                .route("/delete", post(plugins::delete)),
        );
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
