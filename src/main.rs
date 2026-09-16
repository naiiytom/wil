use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(command) = args.next() else {
        return usage();
    };
    if command != "render" {
        return usage();
    }
    let (Some(pack), Some(output)) = (args.next(), args.next()) else {
        return usage();
    };
    if args.next().is_some() {
        return usage();
    }
    match world_in_layers::render_source_pack(pack, output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("render failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> ExitCode {
    eprintln!("usage: world-in-layers render <source-pack> <output.png>");
    ExitCode::FAILURE
}
