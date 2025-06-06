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
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);

    #[cfg(not(windows))]
    let sep = "/";
    #[cfg(windows)]
    let sep = "\\";

    assert!(
        stderr.contains(&format!(
            "=====================
.{sep}subdir{sep}test.hledger
=====================
  2015-10-16 food
-   expenses:food     $10
+   expenses:food  $10"
        )),
        "{}",
        stderr,
    );
    assert!(
        stderr.contains(&format!(
            "==============
.{sep}test.journal
==============
  2015-10-16 food
-   expenses:food     $10
+   expenses:food  $10"
        )),
        "{}",
        stderr,
    );
}
