mod client;
mod networking;
mod server;

use clap::Clap;

/// The options for command line arguments.
/// Parsed with [`clap`].
#[derive(Clap)]
struct Options {
    #[clap(short, long)]
    server: bool,
    #[clap(short, long, default_value = "framefactorserver.ddns.net")]
    ip: String,
}

fn main() {
    let opts = Options::parse();

    if opts.server {
        server::run(opts.ip);
    } else {
        client::run(opts.ip);
    }
}
