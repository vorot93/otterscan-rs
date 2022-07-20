use axum::{
    body::{boxed, Full},
    handler::Handler,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::{get, Router},
    Json,
};
use rust_embed::RustEmbed;
use serde::Serialize;
use std::{marker::PhantomData, net::SocketAddr};
use tracing::*;

pub async fn run(listen_address: SocketAddr, rpc_url: String) -> Result<(), hyper::Error> {
    // Define our app routes, including a fallback option for anything not matched.
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route(
            "/config.json",
            get(move || async move {
                #[derive(Serialize)]
                struct S {
                    #[serde(rename = "erigonURL")]
                    rpc_url: String,
                    #[serde(rename = "assetsURLPrefix")]
                    assets_url_prefix: String,
                }

                Json(S {
                    rpc_url: rpc_url.clone(),
                    assets_url_prefix: format!("http://{listen_address}"),
                })
            }),
        )
        .route("/block/*file", get(index_handler))
        .route("/address/*file", get(index_handler))
        .route("/static/*file", static_handler::<Asset>.into_service())
        .route("/chains/*file", static_handler::<Chains>.into_service())
        .route("/manifest.json", static_handler::<Asset>.into_service())
        .route("/favicon.ico", static_handler::<Asset>.into_service())
        .fallback(get(not_found));

    // Start listening on the given address.
    info!("Otterscan running at http://{listen_address}");
    axum::Server::bind(&listen_address)
        .serve(app.into_make_service())
        .await
}

// We use static route matchers ("/" and "/index.html") to serve our home
// page.
async fn index_handler() -> impl IntoResponse {
    static_handler::<Asset>("/index.html".parse::<Uri>().unwrap()).await
}

// We use a wildcard matcher ("/static/*file") to match against everything
// within our defined assets directory. This is the directory on our Asset
// struct below, where folder = "examples/public/".
async fn static_handler<A: RustEmbed>(uri: Uri) -> impl IntoResponse {
    StaticFile::<A, _>::new(uri.path().trim_start_matches('/').to_string())
}

// Finally, we use a fallback route for anything that didn't match.
async fn not_found() -> Html<&'static str> {
    Html("<h1>404</h1><p>Not Found</p>")
}

#[derive(RustEmbed)]
#[folder = "src/otterscan/build"]
struct Asset;

#[derive(RustEmbed)]
#[folder = "src/otterscan/chains/_data"]
struct Chains;

pub struct StaticFile<A, T> {
    path: T,
    _marker: PhantomData<A>,
}

impl<A, T> StaticFile<A, T> {
    pub fn new(path: T) -> Self {
        Self {
            path,
            _marker: PhantomData,
        }
    }
}

impl<A, T> IntoResponse for StaticFile<A, T>
where
    A: RustEmbed,
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.path.into();

        match A::get(path.as_str()) {
            Some(content) => {
                let body = boxed(Full::from(content.data));
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Full::from("404")))
                .unwrap(),
        }
    }
}
