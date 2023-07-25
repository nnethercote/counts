// Times on a 120MB file:
// - HashMap: 2.6s
// - FnvHashMap: 2.4s
// - FxHashMap: 2.4s
//
// I am partial to FxHashMap, so I chose it over FnvHashMap.

use fxhash::FxHashMap;
use regex_lite::Regex;
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::ops::AddAssign;
use std::str::FromStr;

use Weight::*;

enum Weight {
    Unit,
    Integral,
    Fractional,
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

const USAGE: &str = "\
counts: a tool for ad hoc profiling

USAGE:
    counts [OPTIONS] [FILES]

OPTIONS:
    -h, --help     Print help information
    -i, -w         Integral weighting of lines
    -f             Fractional weighting of lines
    -e             Erase weights after applying, replacing them with `NNN`
";

fn do_main() -> io::Result<()> {
    // Process args.
    let mut weights = Unit;
    let mut erase = false;
    let mut readers: Vec<Box<dyn io::BufRead>> = vec![];
    for arg in env::args().skip(1) {
        if arg == "-h" || arg == "--help" {
            println!("{}", USAGE);
            return Ok(());
        } else if arg == "-i" || arg == "-w" {
            weights = Integral;
        } else if arg == "-f" {
            weights = Fractional;
        } else if arg == "-e" {
            erase = true;
        } else if arg.starts_with('-') {
            eprintln!("counts: unknown option `{}`", arg);
            std::process::exit(1);
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

    let erased_label = if erase { ", erased" } else { "" };
    match weights {
        Unit => process(readers, "", |line| (line, 1i64)),
        Integral => {
            let re = Regex::new(r"(([+-]?)\d+)(\D*)$").unwrap();
            process(
                readers,
                &format!(" (weighted integral{})", erased_label),
                |line| {
                    if let Some(captures) = re.captures(&line) {
                        let weight = i64::from_str(&captures[1]).unwrap();
                        let line = if erase {
                            re.replace(&line, "NNN${3}").to_string()
                        } else {
                            line
                        };
                        (line, weight)
                    } else {
                        (line, 1i64)
                    }
                },
            )
        }
        Fractional => {
            let re = Regex::new(r"(([+-]?)\d+(\.\d+)?)(\D*)$").unwrap();
            process(
                readers,
                &format!(" (weighted fractional{})", erased_label),
                |line| {
                    if let Some(captures) = re.captures(&line) {
                        let weight = f64::from_str(&captures[1]).unwrap();
                        let line = if erase {
                            re.replace(&line, "NNN${4}").to_string()
                        } else {
                            line
                        };
                        (line, weight)
                    } else {
                        (line, 1f64)
                    }
                },
            )
        }
    }
}

// `N` is either `i64` or `f64`, and `f64` values are always of the form
// `mm.nn` so NaNs can't occur and the `PartialOrd` is actually infallible.
fn process<F, N>(
    readers: Vec<Box<dyn BufRead>>,
    label: &str,
    get_line_and_weight: F,
) -> io::Result<()>
where
    F: Fn(String) -> (String, N),
    N: Total,
{
    let mut counts: FxHashMap<String, N> = FxHashMap::default();
    let mut total = N::from(0u32);

    for reader in readers {
        for line in reader.lines() {
            let line = line.unwrap();
            let (line, weight) = get_line_and_weight(line);
            let entry = counts.entry(line).or_insert_with(|| N::from(0u32));
            *entry += weight;
            total += weight;
        }
    }

    // Sort from highest count to lowest count. For lines with the same count,
    // sort them in alphabetical order.
    let mut counts: Vec<_> = counts.iter().collect();
    counts.sort_unstable_by(|(line1, n1), (line2, n2)| {
        (n2.abs(), line1).partial_cmp(&(n1.abs(), line2)).unwrap()
    });

    writeln!(io::stdout(), "{:.1} counts{}", total, label)?;
    let mut cum_perc: f64 = 0f64;
    let total_f64 = total.into_f64();
    for (i, (line, &weight)) in counts.iter().enumerate() {
        let perc: f64 = weight.into_f64() * 100f64 / total_f64;
        cum_perc += perc;
        writeln!(
            io::stdout(),
            "({:3}) {:8.1} ({:4.1}%,{:5.1}%): {}",
            i + 1,
            weight,
            perc,
            cum_perc,
            line
        )?;
    }

    Ok(())
}

trait Total: AddAssign + Copy + Display + From<u32> + PartialOrd {
    /// `f64` doesn't impl `From<i64>` or `TryFrom<i64>`, so we do it
    /// ourselves. We are unlikely to see `i64` values that are so big that
    /// they cannot be represented as `f64`s, so we make this infallible.
    fn into_f64(self) -> f64;

    fn abs(self) -> Self;
}

impl Total for f64 {
    fn into_f64(self) -> f64 {
        self
    }

    fn abs(self) -> f64 {
        self.abs()
    }
}

impl Total for i64 {
    fn into_f64(self) -> f64 {
        let f = self as f64;
        if f as i64 != self {
            panic!("i64 too big to convert to f64")
        }
        f
    }

    fn abs(self) -> i64 {
        self.abs()
    }
}
