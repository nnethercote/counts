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
fn simple() -> Result<(), Box<dyn std::error::Error>> {
    // Each test is a triple:
    //   (input, expected_counts_output, expected_counts_w_output)
    let mut tests = vec![];

    // Empty input.
    tests.push((
        "",
        "\
0 counts:
",
        "\
0 counts:
",
    ));

    // Simple input.
    tests.push((
        "\
a 1
b 2
c 3
d 4
d 4
c 3
c 3
d 4
b 2
d 4
",
        "\
10 counts:
(  1)        4 (40.0%, 40.0%): d 4
(  2)        3 (30.0%, 70.0%): c 3
(  3)        2 (20.0%, 90.0%): b 2
(  4)        1 (10.0%,100.0%): a 1
",
        "\
30 counts:
(  1)       16 (53.3%, 53.3%): d 4
(  2)        9 (30.0%, 83.3%): c 3
(  3)        4 (13.3%, 96.7%): b 2
(  4)        1 ( 3.3%,100.0%): a 1
",
    ));

    for test in tests {
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", test.0)?;

        let mut cmd = Command::cargo_bin("counts")?;
        cmd.arg(file.path());
        cmd.assert().success().stdout(predicate::eq(test.1));

        let mut cmd = Command::cargo_bin("counts")?;
        cmd.arg("-w");
        cmd.arg(file.path());
        cmd.assert().success().stdout(predicate::eq(test.2));
    }

    Ok(())
}
