use axum::http::Response;
use axum::http::{
    header::{HeaderMap, HeaderValue},
    status::StatusCode,
};
use axum::{body::Body, response::IntoResponse, routing::get, Router, Server};
use color_eyre::{eyre::eyre, Report};
use http_body::combinators::UnsyncBoxBody;
use serde::Serialize;
use std::{error::Error, net::SocketAddr};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;
mod tracing_stuff;
mod youtube;
struct ReportError(Report);

impl From<Report> for ReportError {
    fn from(err: Report) -> Self {
        ReportError(err)
    }
}

impl IntoResponse for ReportError {
    fn into_response(self) -> Response<UnsyncBoxBody> {
        // {:?} shows the backtrace / spantrace, see
        // https://docs.rs/eyre/0.6.7/eyre/struct.Report.html#display-representations
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal server error: {:?}", self.0),
        )
            .into_response()
    }
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    tracing_stuff::setup()?;
    run_server().await?;
    tracing_stuff::teardown();
    Ok(())
}

async fn run_server() -> Result<(), Box<dyn Error>> {
    let addr: SocketAddr = "0.0.0.0:3779".parse()?;
    info!("Listening on http://{}", addr);
    let app = Router::new().route("/", get(root)).layer(
        ServiceBuilder::new()
            // 👇 new!
            .layer(TraceLayer::new_for_http())
            .into_inner(),
    );
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

#[tracing::instrument]
async fn root() -> Result<impl IntoResponse, ReportError> {
    let res = youtube::fetch_video_id().await?;
    Ok(res)
}
