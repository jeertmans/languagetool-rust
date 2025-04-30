use std::path::PathBuf;

use assert_cmd::Command;
use predicates::{
    boolean::OrPredicate,
    str::{contains, is_empty, is_match},
};

lazy_static::lazy_static! {
    static ref PATH_ROOT: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    static ref PATH_SAMPLE_FILES: PathBuf = PATH_ROOT.join("tests").join("sample_files");
}

const PATH_FILTERS: [(&str, &str); 1] = [(r" --> .*[\/].*\n", " --> [path]\n")];
macro_rules! assert_snapshot {
    ($label: expr, $snap: expr) => {
        insta::with_settings!({filters => PATH_FILTERS}, {
            insta::assert_snapshot!(
                $label,
                $snap
            );
        });
    };
}

fn get_cmd() -> Command {
    let mut cmd = Command::cargo_bin("ltrs").unwrap();
    cmd.args(["--hostname", "http://localhost", "--port", "8010"]);
    cmd
}

#[test]
fn test_basic_check_text() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .assert();
    assert.success();
}

#[test]
fn test_basic_check_no_errors() {
    let assert = get_cmd()
        .arg("check")
        .arg("-l")
        .arg("en-US")
        .arg("-t")
        .arg("\"I am a star.\"")
        .assert();
    assert
        .success()
        .stdout(contains("No errors were found in provided text"));
}

#[test]
fn test_basic_check_empty_text() {
    let assert = get_cmd().arg("check").arg("--text=").assert();
    assert
        .success()
        .stderr(is_match(r".*WARN.* No input text was provided, skipping.").unwrap());
}

#[test]
fn test_basic_check_data() {
    let assert = get_cmd()
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
    let assert = get_cmd()
        .arg("check")
        .arg("-d")
        .arg("\"some text that is given as text\"")
        .assert();
    assert.failure().stderr(contains("invalid value"));
}

#[test]
fn test_basic_check_wrong_data_2() {
    let assert = get_cmd().arg("check").arg("-d").arg("\"{}\"").assert();
    assert.failure().stderr(contains("invalid value"));
}

#[test]
fn test_basic_check_wrong_data_3() {
    let assert = get_cmd()
        .arg("check")
        .arg("-d")
        .arg("\"some text that is given as text\"")
        .assert();
    assert.failure().stderr(contains("invalid value"));
}

#[test]
fn test_basic_check_piped() {
    let assert = get_cmd()
        .arg("check")
        .write_stdin("some text that is written to stdin")
        .assert();
    assert.success();
}

#[test]
fn test_basic_check_stdin_verbose() {
    let assert = get_cmd()
        .arg("check")
        .arg("-v")
        .arg("-l")
        .arg("en-US")
        .write_stdin("I am a starr.")
        .assert();
    // We only write if terminal is TTY
    assert.success().stderr(is_empty());
}

#[test]
fn test_basic_check_file() {
    use std::io::Write;

    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "Some text with a error inside.").unwrap();

    let assert = get_cmd()
        .arg("check")
        .arg(file.path().to_str().unwrap())
        .assert();
    assert.success();
}

#[test]
fn test_basic_check_files() {
    use std::io::Write;

    let mut file1 = tempfile::NamedTempFile::new().unwrap();
    writeln!(file1, "Some text with a error inside.").unwrap();

    let mut file2 = tempfile::NamedTempFile::new().unwrap();
    writeln!(file2, "Another text with an eror.").unwrap();

    let assert = get_cmd()
        .arg("check")
        .arg(file1.path().to_str().unwrap())
        .arg(file2.path().to_str().unwrap())
        .assert();
    assert.success();
}

#[test]
fn test_basic_check_files_with_empty_file() {
    use std::io::Write;

    let mut file1 = tempfile::NamedTempFile::new().unwrap();
    writeln!(file1, "Some text with a error inside.").unwrap();

    let file2 = tempfile::NamedTempFile::new().unwrap();

    let assert = get_cmd()
        .arg("check")
        .arg("-v")
        .arg(file1.path().to_str().unwrap())
        .arg(file2.path().to_str().unwrap())
        .assert();
    assert
        .success()
        .stderr(is_match(r".*INFO.* Skipping empty file: ").unwrap());
}

#[test]
fn test_basic_check_unexisting_file() {
    let assert = get_cmd()
        .arg("check")
        .arg("some_file_path_that_should_not_exist.txt")
        .assert();
    assert.failure().stderr(contains("invalid filename"));
}

#[test]
fn test_check_with_language() {
    let assert = get_cmd()
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
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("-l")
        .arg("lang")
        .assert();
    assert.failure().stderr(contains("invalid value"));
}

#[test]
fn test_check_with_unexisting_language() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("-l")
        .arg("en-FR")
        .assert();
    assert
        .failure()
        .stderr(contains("not a language code known"));
}

#[test]
fn test_check_with_username_and_key() {
    // TODO: remove the "invalid request" predicate as of LT 6.0
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--username")
        .arg("user")
        .arg("--api-key")
        .arg("key")
        .assert();
    assert.failure().stderr(OrPredicate::new(
        contains("AuthException"),
        contains("invalid request"),
    ));
}

#[test]
fn test_check_with_username_only() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--username")
        .arg("user")
        .assert();
    assert.failure().stderr(contains(
        "the following required arguments were not provided",
    ));
}

#[test]
fn test_check_with_key_only() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--api-key")
        .arg("key")
        .assert();
    assert.failure().stderr(contains(
        "the following required arguments were not provided",
    ));
}

#[test]
fn test_check_with_dict() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--dicts")
        .arg("my_dict")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_dicts() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--dicts")
        .arg("my_dict1,my_dict2")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_preferred_variant() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--preferred-variants")
        .arg("en-GB")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_preferred_variants() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--preferred-variants")
        .arg("en-GB,de-AT")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_language_and_preferred_variant() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("-l")
        .arg("en-US")
        .arg("--preferred-variants")
        .arg("en-GB")
        .assert();
    assert.failure().stderr(contains(
        "the argument \'--language <LANGUAGE>\' cannot be used with \'--preferred-variants \
         <PREFERRED_VARIANTS>\'",
    ));
}

#[test]
fn test_check_with_enabled_rule() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--enabled-rules")
        .arg("EMPTY_LINE")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_enabled_rules() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--enabled-rules")
        .arg("EMPTY_LINE,WHITESPACE_RULE")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_disabled_rule() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--disabled-rules")
        .arg("EMPTY_LINE")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_disabled_rules() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--disabled-rules")
        .arg("EMPTY_LINE,WHITESPACE_RULE")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_enabled_category() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--enabled-categories")
        .arg("STYLE")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_enabled_categories() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--enabled-categories")
        .arg("STYLE,TYPOGRAPHY")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_disabled_category() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--disabled-categories")
        .arg("STYLE")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_disabled_categories() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--disabled-categories")
        .arg("STYLE,TYPOGRAPHY")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_enabled_only_rule() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--enabled-rules")
        .arg("EMPTY_LINE")
        .arg("--enabled-only")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_enabled_only_category() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--enabled-categories")
        .arg("STYLE")
        .arg("--enabled-only")
        .assert();
    assert.success();
}

#[test]
fn test_check_with_enabled_only_without_enabled() {
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--enabled-only")
        .assert();
    assert.failure().stderr(contains("invalid request"));
}

#[test]
fn test_check_with_picky_level() {
    let assert = get_cmd()
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
    let assert = get_cmd()
        .arg("check")
        .arg("-t")
        .arg("\"some text that is given as text\"")
        .arg("--level")
        .arg("strict")
        .assert();
    assert.failure();
}

#[test]
fn test_languages() {
    let assert = get_cmd().arg("languages").assert();
    assert.success();
}

#[test]
fn test_ping() {
    let assert = get_cmd().arg("ping").assert();
    assert.success().stdout(contains("PONG! Delay: "));
}

#[test]
fn test_words() {
    // TODO: remove the "invalid request" predicate as of LT 6.0
    let assert = get_cmd()
        .arg("words")
        .arg("--username")
        .arg("user")
        .arg("--api-key")
        .arg("key")
        .assert();
    assert.failure().stderr(OrPredicate::new(
        contains("AuthException"),
        contains("invalid request"),
    ));
}

#[test]
fn test_words_add() {
    // TODO: remove the "invalid request" predicate as of LT 6.0
    let assert = get_cmd()
        .arg("words")
        .arg("add")
        .arg("--username")
        .arg("user")
        .arg("--api-key")
        .arg("key")
        .arg("my-word")
        .assert();
    assert.failure().stderr(OrPredicate::new(
        contains("AuthException"),
        contains("invalid request"),
    ));
}

#[test]
fn test_words_delete() {
    let assert = get_cmd()
        .arg("words")
        .arg("delete")
        .arg("--username")
        .arg("user")
        .arg("--api-key")
        .arg("key")
        .arg("my-word")
        .assert();
    assert.failure().stderr(OrPredicate::new(
        contains("AuthException"),
        contains("invalid request"),
    ));
}

#[cfg_attr(not(feature = "snapshots"), ignore)]
#[test]
fn test_check_file_typst() {
    let output = get_cmd()
        .arg("check")
        .arg(PATH_SAMPLE_FILES.join("example.typ"))
        .output()
        .unwrap();
    assert_snapshot!(
        "autodetect_typst_file",
        String::from_utf8(output.stdout).unwrap()
    );
}

#[cfg_attr(not(feature = "snapshots"), ignore)]
#[test]
fn test_check_file_html() {
    let output = get_cmd()
        .arg("check")
        .arg(PATH_SAMPLE_FILES.join("example.html"))
        .output()
        .unwrap();
    assert_snapshot!(
        "autodetect_html_file",
        String::from_utf8(output.stdout).unwrap()
    );
}

#[cfg_attr(not(feature = "snapshots"), ignore)]
#[test]
fn test_check_file_markdown() {
    let output = get_cmd()
        .arg("check")
        .arg(PATH_ROOT.join("README.md"))
        .output()
        .unwrap();
    assert_snapshot!(
        "autodetect_markdown_file",
        String::from_utf8(output.stdout).unwrap()
    );
}
