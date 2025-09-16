use ctor::ctor;
use tempfile::TempDir;

#[cfg(not(windows))]
static EXECUTABLE_PATH: &str = "target/debug/hledger-fmt";
#[cfg(windows)]
static EXECUTABLE_PATH: &str = "target\\debug\\hledger-fmt.exe";

#[ctor]
/// Check that the CLI is built and located at `./target/debug/hledger-fmt`.
///
/// This function only runs once, at the start of the test suite.
unsafe fn check_cli_is_built() {
    if !std::path::Path::new(EXECUTABLE_PATH).exists() {
        panic!("CLI not built. Run `cargo build` to build the hledger-fmt debug executable!\n");
    }

    // Check that the executable has been compiled with the `diff` feature enabled.
    let mut cmd = build_cmd();
    let output = cmd.arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("--no-diff") {
        panic!("The CLI executable must be built with the `diff` feature enabled.\n\
                Run `cargo build` to build the hledger-fmt debug executable with the `diff` feature enabled!\n");
    }
}

fn build_cmd() -> assert_cmd::Command {
    let current_source_file = std::path::absolute(file!()).unwrap();
    let target_bin = current_source_file
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(EXECUTABLE_PATH);
    assert_cmd::Command::new(&target_bin)
}

fn tempdir() -> TempDir {
    TempDir::new().unwrap()
}

fn init_cmd(dir: &TempDir) -> assert_cmd::Command {
    let mut cmd = build_cmd();
    cmd.current_dir(dir.path());
    cmd
}

fn assert_contains_journal(value: &str, expected_journal_substring: &str) {
    let normalized_value = value.replace(std::path::MAIN_SEPARATOR_STR, "/");
    assert!(
        normalized_value.contains(expected_journal_substring),
        "Expected to find journal substring:\n\
         --------------------\n\
         {}\n\
         --------------------\n\
         in value:\n\
         --------------------\n\
         {}\n\
         --------------------",
        expected_journal_substring,
        normalized_value
    );
}

/// When no argument is passed and there are no journal files in the current
/// directory nor its subdirectories, hledger-fmt prints help and exits with
/// error code.
#[test]
fn no_args_and_no_journals_prints_help() {
    let dir = tempdir();
    let mut cmd = init_cmd(&dir);

    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(
        "No hledger journal files found in the current directory nor its subdirectories."
    ))
}

/// Walks the current directory and its subdirectories to find hledger journal files.
#[test]
fn walks_directory() {
    let dir = tempdir();
    let file = dir.path().join("test.journal");
    std::fs::write(&file, "2015-10-16 food\n  expenses:food     $10\n").unwrap();
    let subdir = dir.path().join("subdir");
    std::fs::create_dir(&subdir).unwrap();
    let subfile = subdir.join("test.hledger");
    std::fs::write(&subfile, "2015-10-16 food\n  expenses:food     $10\n").unwrap();

    let mut cmd = init_cmd(&dir);

    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty(), "{}", stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_contains_journal(&stderr, "=====================
./subdir/test.hledger
=====================
  2015-10-16 food
-   expenses:food     $10
+   expenses:food  $10");

    assert_contains_journal(&stderr, "==============
./test.journal
==============
  2015-10-16 food
-   expenses:food     $10
+   expenses:food  $10");
}

/// `--exit-zero-on-changes` exits with code 0 when there are changes.
#[test]
fn exit_zero_on_changes_with_changes() {
    let dir = tempdir();
    let file = dir.path().join("test.journal");
    std::fs::write(&file, "2015-10-16 food\n  expenses:food     $10\n").unwrap();
    let mut cmd = init_cmd(&dir);
    let cmd = cmd.arg("--exit-zero-on-changes");

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(&stderr, "  2015-10-16 food
-   expenses:food     $10
+   expenses:food  $10
");
}

/// `--no-diff` does not print diff, but formatted content instead.
#[test]
fn no_diff_prints_formatted_content() {
    let dir = tempdir();
    let file = dir.path().join("test.journal");
    std::fs::write(&file, "2015-10-16 food\n  expenses:food     $10\n").unwrap();
    let mut cmd = init_cmd(&dir);
    let cmd = cmd.arg("--no-diff");

    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(&stdout, "2015-10-16 food
  expenses:food  $10
");
}

/// `--no-diff` + `--exit-zero-on-changes`.
#[test]
fn no_diff_and_exit_zero_on_changes() {
    let dir = tempdir();
    let file = dir.path().join("test.journal");
    std::fs::write(&file, "2015-10-16 food\n  expenses:food     $10\n").unwrap();
    let mut cmd = init_cmd(&dir);
    let cmd = cmd.arg("--no-diff").arg("--exit-zero-on-changes");
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(&stdout, "2015-10-16 food
  expenses:food  $10
");
}
