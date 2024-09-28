use clap::Parser;
#[cfg(feature = "color")]
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use std::ffi::OsStr;
use walkdir::WalkDir;

/// Format hlegder's journal files
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Paths of files to format.
    ///
    /// If not passed, hledger-fmt will search for hledger files in the
    /// current directory and its subdirectories (those that have the
    /// extensions '.journal', '.hledger' or '.j').
    ///
    /// If the paths passed are directories, hledger-fmt will search for
    /// hledger files in those directories and their subdirectories.
    ///
    /// STDIN can be read by passing `-` as a file.
    #[arg(num_args(0..))]
    files: Vec<String>,

    /// Fix the files in place.
    ///
    /// If not passed, hledger-fmt will print the diff between the original
    /// and the formatted file. WARNING: this is a potentially destructive
    /// operation, make sure to make a backup of your files or print the diff
    /// first.
    #[arg(short, long)]
    fix: bool,

    #[cfg(feature = "color")]
    /// Do not use colors in the output.
    ///
    /// You can also use the environment variable NO_COLOR to disable colors.
    #[arg(long)]
    no_color: bool,

    /// Do not print the diff between original and formatted files,
    /// but the new formatted content.
    #[arg(long)]
    no_diff: bool,
}

fn main() {
    let args = Args::parse();
    let mut exitcode = 0;

    // if no files, search in current directory and its subdirectories
    let mut files = Vec::new();
    let stdin = if std::env::args().any(|arg| arg == "-") {
        read_stdin()
    } else {
        "".to_string()
    };

    if stdin.is_empty() {
        if args.files.is_empty() {
            if gather_files_from_directory_and_subdirectories(".", &mut files).is_err() {
                exitcode = 1;
            }

            if files.is_empty() {
                eprintln!(
                    "{}",
                    concat!(
                        "No hledger journal files found in the current directory nor its",
                        " subdirectories.\nEnsure that have extensions '.hledger', '.journal'",
                        " or '.j'."
                    )
                );
                exitcode = 1;
                std::process::exit(exitcode);
            }
        } else {
            for file in &args.files {
                let pathbuf = std::path::PathBuf::from(&file);
                if pathbuf.is_dir() {
                    if gather_files_from_directory_and_subdirectories(file, &mut files).is_err() {
                        exitcode = 1;
                    }
                    break;
                } else if pathbuf.is_file() {
                    if let Ok(content) = read_file(file) {
                        files.push((file.clone(), content));
                    } else {
                        exitcode = 1;
                    }
                } else if !pathbuf.exists() {
                    eprintln!("Path '{file}' does not exist.");
                    exitcode = 1;
                } else if pathbuf.is_symlink() {
                    eprintln!("Path '{file}' is a symlink. Symbolic links are not supported.");
                    exitcode = 1;
                }
            }

            if files.is_empty() {
                eprintln!(
                    "No hledger journal files found looking for next files and/or directories: {:?}.\nEnsure that have extensions '.hledger', '.journal' or '.j'.",
                    args.files,
                );
                exitcode = 1;
                std::process::exit(exitcode);
            }
        }
    } else if files.is_empty() {
        files.push(("".into(), stdin));
    } else {
        eprintln!("Cannot read from STDIN and pass files at the same time.");
        exitcode = 1;
        std::process::exit(exitcode);
    }

    #[cfg(feature = "color")]
    let no_color = args.no_color || std::env::var("NO_COLOR").is_ok();
    let mut something_printed = false;
    let n_files = files.len();

    for (file, content) in files {
        // 1. Parse content
        // 2. Format content
        // 3 Contents are the same? OK
        // 3.1 Contents are different? If `--fix` passed, write new
        // 3.2 Contents are different? If `--fix` not passed, print diff

        let parsed_or_err = hledger_fmt::parser::parse_content(&content);
        if let Err(e) = parsed_or_err {
            if !something_printed {
                something_printed = true;
            } else {
                eprintln!();
            }
            eprintln!(
                "{}",
                hledger_fmt::parser::errors::build_error_context(&e, &content, &file)
            );
            exitcode = 1;
            continue;
        }
        let parsed = parsed_or_err.unwrap();
        let formatted = hledger_fmt::formatter::format_content(&parsed);
        if formatted == content {
            continue;
        }

        if args.fix {
            match std::fs::write(&file, &formatted) {
                Ok(_) => {}
                Err(e) => {
                    if !something_printed {
                        something_printed = true;
                    } else {
                        eprintln!();
                    }
                    eprintln!("Error writing file {file}: {e}");
                }
            }
        } else {
            if n_files > 1 {
                if something_printed {
                    eprintln!();
                } else {
                    something_printed = true;
                }
                // only print the file name if there are more than one file
                let separator = "=".repeat(file.len());
                eprintln!("{separator}\n{file}\n{separator}");
            }

            if args.no_diff {
                #[allow(clippy::print_stdout)]
                {
                    print!("{formatted}");
                }
                continue;
            }

            let diff = TextDiff::from_lines(&content, &formatted);
            for change in diff.iter_all_changes() {
                #[cfg(feature = "color")]
                let line = if no_color {
                    match change.tag() {
                        ChangeTag::Delete => format!("- {change}").normal(),
                        ChangeTag::Insert => format!("+ {change}").normal(),
                        ChangeTag::Equal => format!("  {change}").normal(),
                    }
                } else {
                    match change.tag() {
                        ChangeTag::Delete => format!("- {change}").bright_red(),
                        ChangeTag::Insert => format!("+ {change}").bright_green(),
                        ChangeTag::Equal => format!("  {change}").dimmed(),
                    }
                };

                #[cfg(not(feature = "color"))]
                let line = match change.tag() {
                    ChangeTag::Delete => format!("- {change}"),
                    ChangeTag::Insert => format!("+ {change}"),
                    ChangeTag::Equal => format!("  {change}"),
                };

                eprint!("{line}");
            }
        }
    }

    std::process::exit(exitcode);
}

/// Search for hledger files in the passed directory and its subdirectories
fn gather_files_from_directory_and_subdirectories(
    root: &str,
    files: &mut Vec<(String, String)>,
) -> Result<(), ()> {
    let mut error = false;
    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let ext = entry.path().extension();
        if let Some(ext) = ext {
            if [
                OsStr::new("journal"),
                OsStr::new("hledger"),
                OsStr::new("j"),
            ]
            .contains(&ext)
            {
                let file_path = entry.path().to_str().unwrap().to_string();
                let maybe_file_content = read_file(&file_path);
                if let Ok(content) = maybe_file_content {
                    files.push((file_path, content));
                } else {
                    error = true;
                }
            }
        }
    }

    if error {
        Err(())
    } else {
        Ok(())
    }
}

fn read_file(file_path: &str) -> Result<String, ()> {
    match std::fs::read_to_string(file_path) {
        Ok(content) => Ok(content),
        Err(e) => {
            eprintln!("Error reading file {file_path}: {e}");
            Err(())
        }
    }
}

fn read_stdin() -> String {
    let mut buffer = String::new();
    let lines = std::io::stdin().lines();
    for line in lines {
        buffer.push_str(&line.unwrap());
        buffer.push('\n');
    }
    buffer
}
