//! Minimalistic binary for testing the library.
//!
//! # Usage
//! ```bash
//! nbpc_rs <COMMAND> [OPTIONS]
//! ```
//!
//! ```bash
//! ./nbpc_rs dry_run <n> <l> <rel_payload>
//! ./nbpc_rs l <n> <max_l> <rel_payload> <time_limit_secs>
//! ```
use std::{env, time::Instant};

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let cmd = args
        .next()
        .ok_or("provide the command: `dry_run`".to_string())?;

    match cmd.as_str() {
        "dry_run" => {
            let n = args
                .next()
                .ok_or("provide <n>".to_string())?
                .parse::<usize>()
                .map_err(|_| "<n> is not a valid `usize`".to_string())?;
            let l = args
                .next()
                .ok_or("provide <l>".to_string())?
                .parse::<usize>()
                .map_err(|_| "<l> is not a valid `usize`".to_string())?;
            let rel_payload = args
                .next()
                .ok_or("provide <rel_payload>".to_string())?
                .parse()
                .map_err(|_| "<rel_payload> is not a valid `f64`".to_string())?;

            nbpc_rs::dry_run(n, l, rel_payload);
        }
        "l" => {
            let n = args
                .next()
                .ok_or("provide <n>".to_string())?
                .parse::<usize>()
                .map_err(|_| "<n> is not a valid `usize`".to_string())?;
            let max_l = args
                .next()
                .ok_or("provide <max_l>".to_string())?
                .parse::<usize>()
                .map_err(|_| "<max_l> is not a valid `usize`".to_string())?;
            if max_l == 0 || max_l > 64 {
                return Err("`max_l` must be in `1..=64`".to_string());
            }
            let rel_payload = args
                .next()
                .ok_or("provide <rel_payload>".to_string())?
                .parse()
                .map_err(|_| "<rel_payload> is not a valid `f64`".to_string())?;
            let time_limit_secs = args
                .next()
                .ok_or("provide <time_limit_secs>".to_string())?
                .parse::<u64>()
                .map_err(|_| "<time_limit_secs> is not a valid `u64`".to_string())?;

            for l in 1..=max_l {
                let start = Instant::now();
                let mut iterations = 0;
                let mut total_loss = 0.0;
                while start.elapsed().as_secs() < time_limit_secs as u64 {
                    iterations += 1;
                    total_loss += nbpc_rs::coding_loss_simple(n, l, rel_payload);
                }
                println!(
                    "l = {l:2} | loss = {:.4} ({} iters)",
                    total_loss / iterations as f64,
                    iterations
                )
            }
        }
        unknown_cmd => return Err(format!("unknown command {unknown_cmd}")),
    }

    Ok(())
}
