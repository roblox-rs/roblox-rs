mod build;
mod codegen;
mod describe;
mod iter_ext;

use clap::{Parser, Subcommand};
use log::debug;
use std::{env, fs, path::PathBuf};
use walrus::ModuleConfig;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Build {
        wasm_path: PathBuf,

        #[arg(long, short)]
        out: PathBuf,
    },
}

fn main() {
    let args = Cli::parse();
    env_logger::init();
    debug!("{args:?}");

    match args.command {
        Command::Build { wasm_path, out } => {
            let module = ModuleConfig::new()
                .parse(&fs::read(wasm_path).unwrap())
                .expect("idiot?");

            build::build(module, env::current_dir().unwrap().join(out));
        }
    }
}
