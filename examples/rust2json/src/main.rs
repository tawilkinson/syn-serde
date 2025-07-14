// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{
    env, fs,
    io::{self, BufWriter, Write as _},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let (input_path, output_path, compact) = match &*args {
        [compact_flag, input] if compact_flag == "--compact" => (input, None, true),
        [compact_flag, input, output] if compact_flag == "--compact" => (input, Some(output), true),
        [input] => (input, None, false),
        [input, output] => (input, Some(output), false),
        _ => {
            println!("Usage: rust2json [--compact] <input_path> [output_path]");
            println!("  --compact: Generate JSON without span information");
            std::process::exit(1);
        }
    };

    let code = fs::read_to_string(input_path)?;
    let syn_file = syn::parse_file(&code)?;
    
    // Create syn-serde File with comments extracted from source
    let syntax = syn_serde::File::from_syn_with_comments(&syn_file, &code);

    if let Some(output_path) = output_path {
        if compact {
            let mut value = serde_json::to_value(&syntax)?;
            syn_serde::json::remove_spans(&mut value);
            let buf = serde_json::to_string_pretty(&value)?;
            fs::write(output_path, buf)?;
        } else {
            let buf = serde_json::to_string_pretty(&syntax)?;
            fs::write(output_path, buf)?;
        }
    } else {
        let mut stdout = BufWriter::new(io::stdout().lock()); // Buffered because it is written with newline many times.
        if compact {
            let mut value = serde_json::to_value(&syntax)?;
            syn_serde::json::remove_spans(&mut value);
            serde_json::to_writer_pretty(&mut stdout, &value)?;
        } else {
            serde_json::to_writer_pretty(&mut stdout, &syntax)?;
        }
        stdout.flush()?;
    }
    Ok(())
}
