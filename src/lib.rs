use axum::{http::Response, routing::get};
use tower_http::cors::{Any, CorsLayer};
use tower_service::Service;
use worker::{console_log, event, Context, Env, Fetch, HttpRequest, Result, Url};

macro_rules! handler (
    ($name:path) => {
        |req: axum::extract::Request| async {
            let resp = $name(req.try_into().expect("convert request")).await.expect("handler result");
            Into::<axum::http::Response<axum::body::Body>>::into(resp)
        }
    }
);

pub fn make_router() -> axum::Router {
    let app = axum::Router::new().route("/*path", get(handler!(proxy_handler)))
    .layer(CorsLayer::new().allow_origin(Any));
    app
}
#[worker::send]
async fn proxy_handler(req: axum::extract::Request) -> Result<worker::Response> {
    console_log!("{:?}",req.uri().path().replace("/", ""));
    let url = Url::parse(&req.uri().path().replace("/", ""))?;
    console_log!("{:?}",url);

    let mut response = Fetch::Url(url).send().await?;
    let _ = response.headers_mut().append("access-control-allow-origin",
        "*");
    Ok(response)
}
#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>>{
    console_error_panic_hook::set_once();
    match make_router().call(req).await {
        Ok(resp) => Ok(resp),
        Err(_err) =>  Ok(Response::new("body".into()))
    }
   
}

