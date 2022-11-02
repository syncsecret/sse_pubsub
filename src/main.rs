
use clap::Parser;
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{ Request, Response};
use hyper::body::Body;
use hyper::server::conn::http2;
use hyper::service::{ service_fn};
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Network address to bind to
    #[arg(short, long,  default_value = "127.0.0.1")]
    bind: String,

    /// Network port to use
    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1..), default_value_t = 8080)]
    port: u16,

}
fn main() {
    let args = Args::parse();
   match format!("{}:{}",args.bind,args.port).parse::<SocketAddr>() {
       Ok(addr) => {
           println!("{}",addr);
       }
       Err(err) => {
           println!("{}",err);

       }
   }
}
