use clap::Parser;
use std::path::PathBuf;

// `clap::Parser` is a "derive macro": writing `#[derive(Parser)]` above
// this struct makes the compiler generate all the code that reads
// `std::env::args()`, shows `--help`, etc. In Python you'd use `argparse`,
// in C# `System.CommandLine`: same idea, generated at compile time instead
// of at runtime. Careful: `///` comments (doc comments, below) really do
// end up in the `--help` text shown to the user — `//` comments (like this
// one) stay only in the source code.

/// BASE — Bounded Agent System Environment (dummy version)
#[derive(Parser, Debug)]
#[command(name = "base", version, about = "Bounded Agent System Environment (dummy)")]
pub struct Args {
    /// Folder to open as the sandbox: "." for the current one, a path,
    /// or omit the argument to pick the folder from a dedicated screen
    pub path: Option<PathBuf>,
}