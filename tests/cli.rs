use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::io::Write;
use std::process::Command; // Run programs
use tempfile::NamedTempFile;

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("counts")?;

    cmd.arg("no/such/file");
    cmd.assert().failure().stderr(
        predicate::str::contains("No such file or directory") // Linux/Mac output
            .or(predicate::str::contains("The system cannot find")), // Windows output
    );

    Ok(())
}

#[test]
fn empty() -> Result<(), Box<dyn std::error::Error>> {
    let input = "";
    let tests = vec![
        (vec![], "0 counts\n"),
        (vec!["-w"], "0 counts (weighted integral)\n"),
        (vec!["-i"], "0 counts (weighted integral)\n"),
        (vec!["-f"], "0.0 counts (weighted fractional)\n"),
        (vec!["-e"], "0 counts\n"), // no effect
        (vec!["-i", "-e"], "0 counts (weighted integral, erased)\n"),
        (
            vec!["-f", "-e"],
            "0.0 counts (weighted fractional, erased)\n",
        ),
        // The last option wins.
        (vec!["-f", "-i"], "0 counts (weighted integral)\n"),
        (vec!["-i", "-f"], "0.0 counts (weighted fractional)\n"),
    ];

    good_tests(input, tests)
}

#[test]
fn integral() -> Result<(), Box<dyn std::error::Error>> {
    let input = "\
a 1
b 2
c 3
d 8
c 3
c 3
d 4
b 2
d 4
";
    let tests = vec![
        (
            vec![],
            "\
9 counts
(  1)        3 (33.3%, 33.3%): c 3
(  2)        2 (22.2%, 55.6%): b 2
(  3)        2 (22.2%, 77.8%): d 4
(  4)        1 (11.1%, 88.9%): a 1
(  5)        1 (11.1%,100.0%): d 8
",
        ),
        (
            vec!["-i"],
            "\
30 counts (weighted integral)
(  1)        9 (30.0%, 30.0%): c 3
(  2)        8 (26.7%, 56.7%): d 4
(  3)        8 (26.7%, 83.3%): d 8
(  4)        4 (13.3%, 96.7%): b 2
(  5)        1 ( 3.3%,100.0%): a 1
",
        ),
        (
            vec!["-i", "-e"],
            "\
30 counts (weighted integral, erased)
(  1)       16 (53.3%, 53.3%): d NNN
(  2)        9 (30.0%, 83.3%): c NNN
(  3)        4 (13.3%, 96.7%): b NNN
(  4)        1 ( 3.3%,100.0%): a NNN
",
        ),
        (
            vec!["-f"],
            "\
30.0 counts (weighted fractional)
(  1)      9.0 (30.0%, 30.0%): c 3
(  2)      8.0 (26.7%, 56.7%): d 4
(  3)      8.0 (26.7%, 83.3%): d 8
(  4)      4.0 (13.3%, 96.7%): b 2
(  5)      1.0 ( 3.3%,100.0%): a 1
",
        ),
        (
            vec!["-e", "-f"],
            "\
30.0 counts (weighted fractional, erased)
(  1)     16.0 (53.3%, 53.3%): d NNN
(  2)      9.0 (30.0%, 83.3%): c NNN
(  3)      4.0 (13.3%, 96.7%): b NNN
(  4)      1.0 ( 3.3%,100.0%): a NNN
",
        ),
    ];

    good_tests(input, tests)
}

#[test]
fn fractional() -> Result<(), Box<dyn std::error::Error>> {
    let input = "\
abc (41.3%)
abc (17.5%)
def (9.4%)
ghi (3.7%)
def (1.2%)
def (0.1%)
";

    let tests = vec![
        (
            // Nonsensical, because it only parses only the integer after the '.'.
            vec!["-i"],
            "\
22 counts (weighted integral)
(  1)        7 (31.8%, 31.8%): ghi (3.7%)
(  2)        5 (22.7%, 54.5%): abc (17.5%)
(  3)        4 (18.2%, 72.7%): def (9.4%)
(  4)        3 (13.6%, 86.4%): abc (41.3%)
(  5)        2 ( 9.1%, 95.5%): def (1.2%)
(  6)        1 ( 4.5%,100.0%): def (0.1%)
",
        ),
        (
            // Ditto.
            vec!["-i", "-e"],
            "\
22 counts (weighted integral, erased)
(  1)        7 (31.8%, 31.8%): ghi (3.NNN%)
(  2)        5 (22.7%, 54.5%): abc (17.NNN%)
(  3)        4 (18.2%, 72.7%): def (9.NNN%)
(  4)        3 (13.6%, 86.4%): abc (41.NNN%)
(  5)        2 ( 9.1%, 95.5%): def (1.NNN%)
(  6)        1 ( 4.5%,100.0%): def (0.NNN%)
",
        ),
        (
            vec!["-f"],
            "\
73.2 counts (weighted fractional)
(  1)     41.3 (56.4%, 56.4%): abc (41.3%)
(  2)     17.5 (23.9%, 80.3%): abc (17.5%)
(  3)      9.4 (12.8%, 93.2%): def (9.4%)
(  4)      3.7 ( 5.1%, 98.2%): ghi (3.7%)
(  5)      1.2 ( 1.6%, 99.9%): def (1.2%)
(  6)      0.1 ( 0.1%,100.0%): def (0.1%)
",
        ),
        (
            vec!["-f", "-e"],
            "\
73.2 counts (weighted fractional, erased)
(  1)     58.8 (80.3%, 80.3%): abc (NNN%)
(  2)     10.7 (14.6%, 94.9%): def (NNN%)
(  3)      3.7 ( 5.1%,100.0%): ghi (NNN%)
",
        ),
    ];

    good_tests(input, tests)
}

#[test]
fn signed_integral() -> Result<(), Box<dyn std::error::Error>> {
    let input = "\
foo +3
foo 4
foo 5
bar + -9
bar + -10
baz 23 - 2
baz 23 - +1
";

    let tests = vec![(
        vec!["-i", "-e"],
        "\
-4 counts (weighted integral, erased)
(  1)      -19 (475.0%,475.0%): bar + NNN
(  2)       12 (-300.0%,175.0%): foo NNN
(  3)        3 (-75.0%,100.0%): baz 23 - NNN
",
    )];

    good_tests(input, tests)
}

#[test]
fn signed_fractional() -> Result<(), Box<dyn std::error::Error>> {
    let input = "\
foo +3.3
foo 4.4
foo 5.5
bar + -6.6
bar + -7.0
baz 23 - 2
baz 23 - +1
";

    let tests = vec![(
        vec!["-f", "-e"],
        "\
2.6 counts (weighted fractional, erased)
(  1)    -13.6 (-523.1%,-523.1%): bar + NNN
(  2)     13.2 (507.7%,-15.4%): foo NNN
(  3)      3.0 (115.4%,100.0%): baz 23 - NNN
",
    )];

    good_tests(input, tests)
}

#[test]
fn non_utf8() -> Result<(), Box<dyn std::error::Error>> {
    let input = unsafe { std::str::from_utf8_unchecked(&[0x97, 0x98, 0x99, 0xff]) };
    let expected_output = "counts: stream did not contain valid UTF-8\n";

    bad_test(input, expected_output)
}

fn good_tests(
    input: &'static str,
    tests: Vec<(Vec<&'static str>, &'static str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    for (options, expected_output) in tests {
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", input)?;

        let mut cmd = Command::cargo_bin("counts")?;
        cmd.arg(file.path());
        for option in options {
            cmd.arg(option);
        }
        cmd.assert()
            .success()
            .stdout(predicate::eq(expected_output));
    }
    Ok(())
}

fn bad_test(
    input: &'static str,
    expected_output: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    write!(file, "{}", input)?;

    let mut cmd = Command::cargo_bin("counts")?;
    cmd.arg(file.path());
    cmd.assert()
        .failure()
        .stderr(predicate::eq(expected_output));
    Ok(())
}
