use assert_cmd::Command;

#[test]
fn test_basic_check_text() {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .assert();
    assert.success();
}

#[test]
fn test_basic_check_data() {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd
        .arg("check")
        .arg("-d")
        .arg(
            r#"{"annotation":[{"text": "A "},{"markup": "<b>"},{"text": "test"},{"markup": "</b>"}
]}"#,
        )
        .assert();
    assert.success();
}

#[test]
fn test_basic_check_wrong_data_1() {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd
        .arg("check")
        .arg("-d")
        .arg("\"some text that is given as text\"")
        .assert();
    assert.failure();
}

#[test]
fn test_basic_check_wrong_data_2() {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd.arg("check").arg("-d").arg("\"{}\"").assert();
    assert.failure();
}

#[test]
fn test_basic_check_wrong_data_3() {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd
        .arg("check")
        .arg("-d")
        .arg("\"some text that is given as text\"")
        .assert();
    assert.failure();
}

#[test]
fn test_basic_check_piped() {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd
        .arg("check")
        .write_stdin("some text that is piped")
        .assert();
    assert.success();
}

#[test]
fn test_basic_check_file() {
    use std::io::Write;

    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "Some text with a error inside.").unwrap();

    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd.arg("check").arg(file.path().to_str().unwrap()).assert();
    assert.success();
}

#[test]
fn test_basic_check_files() {
    use std::io::Write;

    let mut file1 = tempfile::NamedTempFile::new().unwrap();
    writeln!(file1, "Some text with a error inside.").unwrap();

    let mut file2 = tempfile::NamedTempFile::new().unwrap();
    writeln!(file2, "Another text with an eror.").unwrap();

    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd
        .arg("check")
        .arg(file1.path().to_str().unwrap())
        .arg(file2.path().to_str().unwrap())
        .assert();
    assert.success();
}

#[test]
fn test_basic_check_unexisting_file() {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd
        .arg("check")
        .arg("some_file_path_that_should_not_exist.txt")
        .assert();
    assert.failure();
}

#[test]
fn test_check_with_language() {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("-l")
        .arg("en-US")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_wrong_language() {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("-l")
        .arg("lang")
        .assert();
    assert.failure();
}

#[test]
fn test_check_with_unexisting_language() {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("-l")
        .arg("en-FR")
        .assert();
    assert.failure();
}

#[test]
fn test_check_with_picky_level() {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--level")
        .arg("picky")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_unexisting_level() {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    let assert = cmd
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--level")
        .arg("strict")
        .assert();
    assert.failure();
}
