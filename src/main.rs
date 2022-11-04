use clap::Parser;

/// SSE pubsub broker
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
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;
type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

static INDEX: &[u8] = b"<a href=\"test.html\">test.html</a>";
static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";
static POST_DATA: &str = r#"{"original": "data"}"#;
static URL: &str = "http://127.0.0.1:1337/json_api";
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Client {
    id: Uuid,
}

impl Client {
    pub fn new() -> Client {
        Client { id: Uuid::new_v4() }
    }
}

enum Route {
    NotFound,
    Publish(Body),
    Stats,
    Subscribe,
}
#[derive(Clone, Copy)]
enum Body {
    Len(u64),
    Chunked,
}

async fn subscribe_response() -> Result<Response<BoxBody>> {
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

async fn publish_response(req: Request<IncomingBody>) -> Result<Response<BoxBody>> {
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

async fn response_examples(req: Request<IncomingBody>) -> Result<Response<BoxBody>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full(INDEX))),
        (&Method::GET, "/publish") => subscribe_response().await,
        (&Method::POST, "/subscribe") => publish_response(req).await,
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
            let local = tokio::task::LocalSet::new();

            loop {
                let (stream, _) = listener.accept().await?;

                local.run_until(async {
                    local.spawn_local(async {
                        let service = service_fn(response_examples);
                        println!("in here");
                        if let Err(err) = http2::Builder::new(LocalExec)
                            .serve_connection(stream, service)
                            .await
                        {
                            println!("Failed to serve connection: {:?}", err);
                        }
                    }).await.unwrap();
                }).await;
            }
        }
        Err(err) => {
            println!("{}", err);
            Ok(())
        }
    }
}
// NOTE: This part is only needed for HTTP/2. HTTP/1 doesn't need an executor.
//
// Since the Server needs to spawn some background tasks, we needed
// to configure an Executor that can spawn !Send futures...
#[derive(Clone, Copy, Debug)]
struct LocalExec;

impl<F> hyper::rt::Executor<F> for LocalExec
    where
        F: std::future::Future + 'static, // not requiring `Send`
{
    fn execute(&self, fut: F) {
        println!("in execture");
        // This will spawn into the currently running `LocalSet`.
        tokio::task::spawn_local(fut);
    }
}