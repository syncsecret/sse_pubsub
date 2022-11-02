
use clap::Parser;

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

}
