use assert_cmd::Command;
use indoc::indoc;

const BIN_NAME: &str = "tetris";
const EXAMPLE1: &str = "I0,I4,Q8";
const EXAMPLE2: &str = "T1,Z3,I4";
const EXAMPLE3: &str = "Q0,I2,I6,I0,I6,I6,Q2,Q4";

#[test]
fn example1() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .write_stdin(EXAMPLE1)
        .assert()
        .success()
        .stdout("1\n");
    Ok(())
}

#[test]
fn example2() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .write_stdin(EXAMPLE2)
        .assert()
        .success()
        .stdout("4\n");
    Ok(())
}

#[test]
fn example3() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .write_stdin(EXAMPLE3)
        .assert()
        .success()
        .stdout("3\n");
    Ok(())
}

#[test]
fn all_examples() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .write_stdin(format!("{EXAMPLE1}\n{EXAMPLE2}\n{EXAMPLE3}"))
        .assert()
        .success()
        .stdout("1\n4\n3\n");
    Ok(())
}

// todo: answer not provided
#[test]
fn given_input_txt_parses() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .write_stdin(indoc!(
            "
            Q0
            Q0,Q1
            Q0,Q2,Q4,Q6,Q8
            Q0,Q2,Q4,Q6,Q8,Q1
            Q0,Q2,Q4,Q6,Q8,Q1,Q1
            I0,I4,Q8
            I0,I4,Q8,I0,I4
            L0,J2,L4,J6,Q8
            T0,T3
            T0,T3,I6,I6
            I0,I6,S4
            T1,Z3,I4
            L0,J3,L5,J8,T1
            L0,J3,L5,J8,T1,T6
            L0,J3,L5,J8,T1,T6,J2,L6,T0,T7
            L0,J3,L5,J8,T1,T6,J2,L6,T0,T7,Q4
            S0,S2,S4,S6
            S0,S2,S4,S5,Q8,Q8,Q8,Q8,T1,Q1,I0,Q4
            L0,J3,L5,J8,T1,T6,S2,Z5,T0,T7
            Q0,I2,I6,I0,I6,I6,Q2,Q4
        "
        ))
        .assert()
        .success();
    Ok(())
}
