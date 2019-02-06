// Times on a 120MB file:
// - HashMap: 2.6s
// - FnvHashMap: 2.4s
// - FxHashMap: 2.4s
//
// I am partial to FxHashMap, so I chose it over FnvHashMap.

extern crate fxhash;
extern crate regex;

use fxhash::FxHashMap; // faster than `FnvHashMap` here, oddly enough
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Result};
use std::str::FromStr;

fn main() -> Result<()> {
    // Process args.
    let mut weighted = false;
    let mut readers: Vec<Box<dyn BufRead>> = vec![];
    for arg in env::args().skip(1) {
        if arg == "-w" {
            weighted = true;
        } else if arg.starts_with("-") {
            println!("usage: counts [-w] [infiles ...]");
            return Ok(());
        } else {
            let file = File::open(arg)?;
            let reader = Box::new(BufReader::new(file));
            readers.push(reader);
        }
    }

    // Use stdin if no files were specified.
    if readers.is_empty() {
        readers.push(Box::new(BufReader::new(io::stdin())))
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
    println!("{} counts:", total);
    let mut cum_perc: f64 = 0f64;
    for (i, (line, &weight)) in counts.iter().enumerate() {
        let perc: f64 = (weight as f64) * 100f64 / (total as f64);
        cum_perc += perc;
        println!("({:3}) {:8} ({:4.1}%,{:5.1}%): {}",
                 i + 1, weight, perc, cum_perc, line)
    }

    Ok(())
}
