#![feature(async_closure)]

use std::net::SocketAddr;
use std::str::FromStr;

use axum::{
    routing::{get, get_service, post},
    Router,
};
use clap::Parser;
use tower_http::services::ServeDir;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use pbrq::handler::{bot, password, plugins, qrcode};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Bind addr
    #[clap(long, value_parser, default_value = "0.0.0.0:9000")]
    bind_addr: String,

    /// Location of static dir
    #[clap(long, value_parser)]
    static_dir: Option<String>,

    /// Username of basic auth
    #[clap(long, value_parser)]
    basic_username: Option<String>,

    /// Password of basic auth
    #[clap(long, value_parser, default_value = "123456")]
    basic_password: String,

    /// Allow cors
    #[clap(long, value_parser, default_value_t = false)]
    cors: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let addr = SocketAddr::from_str(&args.bind_addr).expect("failed to parse arg: bind_addr");
    init_log();
    let mut app = Router::new()
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
                        .route("/request_sms", post(password::request_sms))
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
    if let Some(static_dir) = args.static_dir.as_ref() {
        tracing::info!("http_static_dir: {}", static_dir);
        app = app.fallback(get_service(ServeDir::new(static_dir)).handle_error(handle_error));
    }
    if let Some(username) = args.basic_username.as_ref() {
        tracing::info!("http_basic_auth: true");
        app = app.layer(tower_http::auth::RequireAuthorizationLayer::basic(
            username,
            &args.basic_password,
        ))
    }
    if args.cors {
        tracing::info!("http_allow_cors: true");
        app = app.layer(tower_http::cors::CorsLayer::permissive())
    }
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn init_log() {
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
                .with_target("pbrq", Level::DEBUG)
                .with_target("ricq", Level::DEBUG),
        )
        .init();
}

async fn handle_error(_: std::io::Error) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "Something went wrong...",
    )
}
