use assert_cmd::Command;

const BINARY_NAME: &str = "forth_times_the_charm";

#[test]
fn if_then_else() {
    make_command()
        .write_stdin("10 20 < if 1 else 0 then .")
        .assert()
        .stdout(predicates::str::diff("1\n"));
}

#[test]
fn equal() {
    make_command()
        .write_stdin("10 10 = .")
        .assert()
        .stdout(predicates::str::diff("1\n"));
}

#[test]
fn not_equal() {
    make_command()
        .write_stdin("10 20 <> .")
        .assert()
        .stdout(predicates::str::diff("1\n"));
}

#[test]
fn less_than() {
    make_command()
        .write_stdin("10 20 < .")
        .assert()
        .stdout(predicates::str::diff("1\n"));
}

#[test]
fn greater_than() {
    make_command()
        .write_stdin("10 20 > .")
        .assert()
        .stdout(predicates::str::diff("0\n"));
}

#[test]
fn define() {
    make_command()
        .write_stdin(": double dup + ; 10 double .")
        .assert()
        .stdout(predicates::str::diff("20\n"));
}

#[test]
fn duplication() {
    make_command()
        .write_stdin("10 dup + .")
        .assert()
        .stdout(predicates::str::diff("20\n"));
}

#[test]
fn division() {
    make_command()
        .write_stdin("20 10 / .")
        .assert()
        .stdout(predicates::str::diff("2\n"));
}

#[test]
fn multiplication() {
    make_command()
        .write_stdin("10 20 * .")
        .assert()
        .stdout(predicates::str::diff("200\n"));
}

#[test]
fn subtraction() {
    make_command()
        .write_stdin("40 10 - .")
        .assert()
        .stdout(predicates::str::diff("30\n"));
}

#[test]
fn addition() {
    make_command()
        .write_stdin("10 20 + .")
        .assert()
        .stdout(predicates::str::diff("30\n"));
}

#[test]
fn it_runs() {
    make_command().assert().success();
}

fn make_command() -> Command {
    let cmd = Command::cargo_bin(BINARY_NAME).unwrap();
    cmd
}
