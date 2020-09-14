// Times on a 120MB file:
// - HashMap: 2.6s
// - FnvHashMap: 2.4s
// - FxHashMap: 2.4s
//
// I am partial to FxHashMap, so I chose it over FnvHashMap.

extern crate fxhash;
extern crate regex;

use fxhash::FxHashMap;
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::str::FromStr;

fn do_main() -> io::Result<()> {
    // Process args.
    let mut weighted = false;
    let mut readers: Vec<Box<dyn io::BufRead>> = vec![];
    for arg in env::args().skip(1) {
        if arg == "-w" {
            weighted = true;
        } else if arg.starts_with('-') {
            println!("usage: counts [-w] [infiles ...]");
            return Ok(());
        } else {
            let file = File::open(arg)?;
            let reader = Box::new(io::BufReader::new(file));
            readers.push(reader);
        }
    }

    // Use stdin if no files were specified.
    if readers.is_empty() {
        readers.push(Box::new(io::BufReader::new(io::stdin())))
    }

    // Initialize.
    let mut counts: FxHashMap<String, u64> = FxHashMap::default();
    let mut total = 0u64;

    // Process inputs.
    let re = Regex::new(r"[^\d](\d+)[^\d]*$").unwrap();
    for reader in readers {
        for line in reader.lines() {
            let line = line.unwrap();

            let mut weight = 1;
            if weighted {
                if let Some(captures) = re.captures(&line) {
                    weight = u64::from_str(&captures[1]).unwrap();
                }
            }

            let entry = counts.entry(line).or_insert(0);
            *entry += weight;
            total += weight;
        }
    }

    // Collect and sort the histogram.
    let mut counts: Vec<_> = counts.iter().collect();
    counts.sort_unstable_by(|(_, n1), (_, n2)| n2.cmp(n1));

    // Print the histogram.
    writeln!(io::stdout(), "{} counts:", total)?;
    let mut cum_perc: f64 = 0f64;
    for (i, (line, &weight)) in counts.iter().enumerate() {
        let perc: f64 = (weight as f64) * 100f64 / (total as f64);
        cum_perc += perc;
        writeln!(
            io::stdout(),
            "({:3}) {:8} ({:4.1}%,{:5.1}%): {}",
            i + 1,
            weight,
            perc,
            cum_perc,
            line
        )?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    // Ignore broken pipes, which occur for `counts input.txt | head -11`.
    match do_main() {
        Ok(_) => Ok(()),
        Err(ref err) if err.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => {
            eprintln!("counts: {}", err);
            std::process::exit(1);
        }
    }
}
