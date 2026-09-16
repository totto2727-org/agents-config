use std::{error::Error, process::Command};

#[test]
fn prints_greeting() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_project")).output()?;

    assert!(output.status.success());
    assert_eq!(std::str::from_utf8(&output.stdout)?, "Hello, world!\n");

    Ok(())
}
