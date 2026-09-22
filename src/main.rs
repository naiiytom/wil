use std::{env, ffi::OsString, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.is_empty() {
        return usage();
    }

    let command = args[0].to_string_lossy().to_string();
    let rest = &args[1..];

    let mut positionals = Vec::new();
    let mut jobs: Option<usize> = None;
    let mut i = 0;

    while i < rest.len() {
        let arg = rest[i].to_string_lossy();
        if arg == "--jobs" || arg == "-j" {
            if i + 1 >= rest.len() {
                eprintln!("error: missing value for {arg}");
                return usage();
            }
            i += 1;
            let val_str = rest[i].to_string_lossy();
            match parse_positive_jobs(&val_str, "--jobs") {
                Ok(n) => jobs = Some(n),
                Err(code) => return code,
            }
        } else if let Some(val_str) = arg.strip_prefix("--jobs=") {
            match parse_positive_jobs(val_str, "--jobs") {
                Ok(n) => jobs = Some(n),
                Err(code) => return code,
            }
        } else if let Some(val_str) = arg.strip_prefix("-j=") {
            match parse_positive_jobs(val_str, "-j") {
                Ok(n) => jobs = Some(n),
                Err(code) => return code,
            }
        } else if arg.starts_with('-') {
            eprintln!("error: unrecognized flag '{arg}'");
            return usage();
        } else {
            positionals.push(&rest[i]);
        }
        i += 1;
    }

    if positionals.len() != 2 {
        return usage();
    }

    let pack = positionals[0];
    let output = positionals[1];

    let result = match command.as_str() {
        "render" => {
            if jobs.is_some() {
                eprintln!("error: --jobs is not supported for single image rendering");
                return usage();
            }
            world_in_layers::render_source_pack(pack, output)
        }
        "render-sequence" => world_in_layers::render_sequence_pack_with_jobs(pack, output, jobs),
        "render-episode" => world_in_layers::render_episode_pack(pack, output, jobs),
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
        "usage: world-in-layers render <source-pack> <output.png>\n       world-in-layers render-sequence <source-pack> <output-dir> [--jobs <N>]\n       world-in-layers render-episode <episode-pack> <output-dir> [--jobs <N>]"
    );
    ExitCode::FAILURE
}

fn parse_positive_jobs(val_str: &str, flag: &str) -> Result<usize, ExitCode> {
    match val_str.parse::<usize>() {
        Ok(n) if n > 0 => Ok(n),
        _ => {
            eprintln!("error: {flag} must be a positive integer");
            Err(ExitCode::FAILURE)
        }
    }
}
