use clap::Parser;
// use std::convert::Infallible;
// use std::future::Future;
// use std::net::SocketAddr;
// use hyper::{Request, Response};
// use hyper::body::Body;
// use hyper::Server;
// use hyper::service::{service_fn};
// use tokio::net::TcpListener;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Network address to bind to
    #[arg(short, long, default_value = "127.0.0.1")]
    bind: String,

    /// Network port to use
    #[arg(short, long, value_parser = clap::value_parser ! (u16).range(1..), default_value_t = 8080)]
    port: u16,

}




use std::net::SocketAddr;

use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};
use tokio::net::{TcpListener, TcpStream};

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;
type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

static INDEX: &[u8] = b"<a href=\"test.html\">test.html</a>";
static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";
static POST_DATA: &str = r#"{"original": "data"}"#;
static URL: &str = "http://127.0.0.1:1337/json_api";

async fn client_request_response() -> Result<Response<BoxBody>> {
    let req = Request::builder()
        .method(Method::POST)
        .uri(URL)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::new(Bytes::from(POST_DATA)))
        .unwrap();

    let host = req.uri().host().expect("uri has no host");
    let port = req.uri().port_u16().expect("uri has no port");
    let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;

    let (mut sender, conn) = hyper::client::conn::http1::handshake(stream).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection error: {:?}", err);
        }
    });

    let web_res = sender.send_request(req).await?;

    let res_body = web_res.into_body().boxed();

    Ok(Response::new(res_body))
}

async fn api_post_response(req: Request<IncomingBody>) -> Result<Response<BoxBody>> {
    // Aggregate the body...
    let whole_body = req.collect().await?.aggregate();
    // Decode as JSON...
    let mut data: serde_json::Value = serde_json::from_reader(whole_body.reader())?;
    // Change the JSON...
    data["test"] = serde_json::Value::from("test_value");
    // And respond with the new JSON.
    let json = serde_json::to_string(&data)?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(full(json))?;
    Ok(response)
}

async fn api_get_response() -> Result<Response<BoxBody>> {
    let data = vec!["foo", "bar"];
    let res = match serde_json::to_string(&data) {
        Ok(json) => Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(full(json))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(full(INTERNAL_SERVER_ERROR))
            .unwrap(),
    };
    Ok(res)
}

async fn response_examples(req: Request<IncomingBody>) -> Result<Response<BoxBody>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => Ok(Response::new(full(INDEX))),
        (&Method::GET, "/test.html") => client_request_response().await,
        (&Method::POST, "/json_api") => api_post_response(req).await,
        (&Method::GET, "/json_api") => api_get_response().await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full(NOTFOUND))
                .unwrap())
        }
    }
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = Args::parse();
    match format!("{}:{}", args.bind, args.port).parse::<SocketAddr>() {
        Ok(addr) => {
            let listener = TcpListener::bind(&addr).await?;
            println!("Listening on http://{}", addr);
            loop {
                let (stream, _) = listener.accept().await?;

                tokio::task::spawn(async move {
                    let service = service_fn(response_examples);

                    if let Err(err) = http1::Builder::new()
                        .serve_connection(stream, service)
                        .await
                    {
                        println!("Failed to serve connection: {:?}", err);
                    }
                });
            }
        }
        Err(err) => {
            println!("{}", err);
            Ok(())
        }
    }



}
