use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(command) = args.next() else {
        return usage();
    };
    let (Some(pack), Some(output)) = (args.next(), args.next()) else {
        return usage();
    };
    if args.next().is_some() {
        return usage();
    }
    let result = match command.to_string_lossy().as_ref() {
        "render" => world_in_layers::render_source_pack(pack, output),
        "render-sequence" => world_in_layers::render_sequence_pack(pack, output),
        _ => return usage(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("render failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> ExitCode {
    eprintln!(
        "usage: world-in-layers render <source-pack> <output.png>\n       world-in-layers render-sequence <source-pack> <output-dir>"
    );
    ExitCode::FAILURE
}
