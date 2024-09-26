use clap::Parser;
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
    /// extensions '.journal', '.hledger' and '.j').
    #[arg(num_args(0..))]
    files: Vec<String>,

    /// Fix the files in place.
    ///
    /// If not passed, hledger-fmt will print the diff between the original
    /// and the formatted file. WARNING: this is a potentially destructive
    /// operation, make sure to have a backup of your files or print the diff
    /// first.
    #[arg(short, long)]
    fix: bool,

    /// Do not use colors in the output.
    ///
    /// You can also use the environment variable NO_COLOR to disable colors.
    #[arg(long)]
    no_color: bool,
}

fn main() {
    let args = Args::parse();

    // if no files, search in current directory and its subdirectories
    let mut files = args.files;
    if files.is_empty() {
        gather_files_from_directory_and_subdirectories(&mut files);
    }

    let no_color = args.no_color || std::env::var("NO_COLOR").is_ok();

    let mut exitcode = 0;
    let mut something_printed = false;

    for file in files {
        let content = match std::fs::read_to_string(&file) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file {}: {}", file, e);
                exitcode = 1;
                continue;
            }
        };

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
            exitcode = 2;
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
                    eprintln!("Error writing file {}: {}", file, e);
                    exitcode = 1;
                }
            }
        } else {
            if !something_printed {
                something_printed = true;
            } else {
                eprintln!();
            }

            let diff = TextDiff::from_lines(&content, &formatted);

            let separator = "=".repeat(file.len());
            eprintln!("{}\n{}\n{}", separator, file, separator);
            for change in diff.iter_all_changes() {
                let line = if no_color {
                    match change.tag() {
                        ChangeTag::Delete => format!("- {}", change).normal(),
                        ChangeTag::Insert => format!("+ {}", change).normal(),
                        ChangeTag::Equal => format!("  {}", change).normal(),
                    }
                } else {
                    match change.tag() {
                        ChangeTag::Delete => format!("- {}", change).bright_red(),
                        ChangeTag::Insert => format!("+ {}", change).bright_green(),
                        ChangeTag::Equal => format!("  {}", change).dimmed(),
                    }
                };
                eprint!("{}", line);
            }
        }
    }

    std::process::exit(exitcode);
}

/// Search for hledger files in the current directory and its subdirectories
fn gather_files_from_directory_and_subdirectories(files: &mut Vec<String>) {
    for entry in WalkDir::new(".")
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
                files.push(entry.path().to_str().unwrap().to_string());
            }
        }
    }
}
