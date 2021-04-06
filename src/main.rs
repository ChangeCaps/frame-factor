//! The Frame Factor code base.

mod angle;
mod animation;
mod attack;
mod camera;
mod client;
mod collider;
mod frame;
mod game_settings;
mod game_state;
mod helper;
mod input;
mod networking;
mod player;
mod progress_bar;
mod server;
mod transform;

use clap::Clap;

/// The options for command line arguments.
/// Parsed with [`clap`].
#[derive(Clap)]
struct Options {
    #[clap(short, long)]
    server: bool,
    #[clap(short, long, default_value = "framefactorserver.ddns.net:35566")]
    ip: String,
    #[clap(short, long)]
    local: bool,
}

fn main() {
    let opts = Options::parse();

    if opts.local {
        std::process::Command::new(std::env::args().next().unwrap())
            .arg("-i")
            .arg(opts.ip.clone())
            .spawn()
            .unwrap();

        std::process::Command::new(std::env::args().next().unwrap())
            .arg("-i")
            .arg(opts.ip.clone())
            .spawn()
            .unwrap();

        server::run(opts.ip);
    } else {
        if opts.server {
            server::run(opts.ip);
        } else {
            client::run(opts.ip);
        }
    }
}
