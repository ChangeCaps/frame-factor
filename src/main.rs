//! The Frame Factor code base.

mod client;
mod frame;
mod game_settings;
mod game_state;
mod helper;
mod input;
mod networking;
mod player;
mod server;
mod world_transform;

use clap::Clap;

/// The options for command line arguments.
/// Parsed with [`clap`].
#[derive(Clap)]
struct Options {
    #[clap(short, long)]
    server: bool,
    #[clap(short, long, default_value = "framefactorserver.ddns.net:35566")]
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
