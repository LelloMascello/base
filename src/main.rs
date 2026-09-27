mod cli;
mod tui;
mod agent;
mod sandbox;
mod config;

use cli::Args;
use clap::Parser;
//fn main() {
//    println!("Hello, world!");
//}
fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello {}!", args.name);
    }
}