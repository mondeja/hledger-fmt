#[doc(hidden)]
pub mod builder;
#[cfg(test)]
mod tests;

use crate::file_path::FilePath;
use std::io::Read;

#[doc(hidden)]
/// Run the hledger-fmt CLI and return the exit code.
pub fn run(cmd: clap::Command) -> i32 {
    let args = cmd.get_matches();
    let files_arg: Vec<String> = if let Some(files) = args.get_many("files") {
        files.cloned().collect()
    } else {
        Vec::new()
    };
    let files_arg: Vec<FilePath> =
        files_arg.iter().map(|s| FilePath::from(s.as_str())).collect();
    let fix = args.get_flag("fix");
    let exit_zero_on_changes = args.get_flag("exit-zero-on-changes");

    #[cfg(feature = "diff")]
    let no_diff = args.get_flag("no-diff");

    let mut exitcode = 0;

    // if no files, search in current directory and its subdirectories
    let mut files: Vec<(FilePath, Vec<u8>)> = Vec::new();
    let stdin = if std::env::args().any(|arg| arg == "-") {
        read_stdin()
    } else {
        Vec::with_capacity(0)
    };

    if stdin.is_empty() {
        if files_arg.is_empty() {
            if gather_files_from_directory_and_subdirectories(&FilePath::from(b'.'), &mut files)
                .is_err()
            {
                exitcode = 1;
            }

            if files.is_empty() {
                eprintln!(
                    "No hledger journal files found in the current directory nor its subdirectories.\n\
                     Ensure that they have extensions '.hledger', '.journal' or '.j'."
                );
                exitcode = 1;
                return exitcode;
            }
        } else {
            for file_path in &files_arg {
                let pathbuf = std::path::PathBuf::from(file_path);
                if pathbuf.is_dir() {
                    if gather_files_from_directory_and_subdirectories(file_path, &mut files)
                        .is_err()
                    {
                        exitcode = 1;
                    }
                    break;
                } else if pathbuf.is_file() {
                    if let Ok(content) = read_file(file_path) {
                        files.push((file_path.clone(), content));
                    } else {
                        exitcode = 1;
                    }
                } else if !pathbuf.exists() {
                    eprintln!("Path '{file_path}' does not exist.");
                    exitcode = 1;
                } else if pathbuf.is_symlink() {
                    eprintln!("Path '{file_path}' is a symlink. Symbolic links are not supported.");
                    exitcode = 1;
                }
            }

            if exitcode != 0 {
                return exitcode;
            }

            if files.is_empty() {
                eprintln!(
                    "No hledger journal files found looking for next files and/or directories: {files_arg:#?}.\n\
                     Ensure that they have extensions '.hledger', '.journal' or '.j'.",
                );
                exitcode = 1;
                return exitcode;
            }
        }
    } else if files.is_empty() {
        files.push((FilePath::new(), stdin));
    } else {
        eprintln!("Cannot read from STDIN and pass files at the same time.");
        exitcode = 1;
        return exitcode;
    }

    let mut something_printed = false;
    let n_files = files.len();

    for (file, content) in files {
        // 1. Parse content
        // 2. Format content
        // 3  Contents are the same?
        // 3.1 YES
        // 3.2 NO
        // 3.2.1 `--fix` passed?
        // 3.2.1.1 YES -> Write new
        // 3.2.1.2 NO  ->  Print diff

        let parsed_or_err = crate::parser::parse_content(&content);
        if let Err(e) = parsed_or_err {
            if !something_printed {
                something_printed = true;
            } else {
                eprintln!();
            }
            eprintln!(
                "{}",
                crate::parser::errors::build_error_context(&e, &content, &file)
            );
            exitcode = 1;
            continue;
        }
        let parsed = parsed_or_err.unwrap();
        let format_opts =
            crate::formatter::FormatContentOptions::new().with_estimated_length(content.len());
        let buffer = crate::formatter::format_content_with_options(&parsed, &format_opts);
        if buffer == content {
            #[cfg(feature = "diff")]
            {
                if !no_diff {
                    continue;
                }
            }

            #[cfg(not(feature = "diff"))]
            continue;
        }

        #[cfg(feature = "diff")]
        {
            if exitcode == 0 && !exit_zero_on_changes {
                exitcode = 2;
            }
        }

        if fix {
            match std::fs::write(&file, &buffer) {
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
                let file_name = file.to_string_lossy();
                let separator = "=".repeat(file_name.chars().count());
                eprintln!("{separator}\n{file}\n{separator}");
            }

            let formatted = String::from_utf8_lossy(&buffer);

            #[cfg(not(feature = "diff"))]
            {
                #[allow(clippy::print_stdout)]
                {
                    print!("{formatted}");
                }
            }

            #[cfg(feature = "diff")]
            {
                use similar::{ChangeTag, TextDiff};

                if no_diff {
                    #[allow(clippy::print_stdout)]
                    {
                        print!("{formatted}");
                    }
                    continue;
                }

                let content_as_str = String::from_utf8_lossy(&content);

                let diff = TextDiff::from_lines(&content_as_str, &formatted);
                for change in diff.iter_all_changes() {
                    #[cfg(not(feature = "color"))]
                    {
                        let line = match change.tag() {
                            ChangeTag::Delete => format!("- {change}"),
                            ChangeTag::Insert => format!("+ {change}"),
                            ChangeTag::Equal => format!("  {change}"),
                        };
                        eprint!("{line}");
                    }

                    #[cfg(feature = "color")]
                    {
                        let line = match change.tag() {
                            ChangeTag::Delete => {
                                let bright_red = anstyle::Style::new()
                                    .fg_color(Some(anstyle::AnsiColor::BrightRed.into()));
                                format!("{bright_red}- {change}{bright_red:#}")
                            }
                            ChangeTag::Insert => {
                                let bright_green = anstyle::Style::new()
                                    .fg_color(Some(anstyle::AnsiColor::BrightGreen.into()));
                                format!("{bright_green}+ {change}{bright_green:#}")
                            }
                            ChangeTag::Equal => {
                                let dimmed = anstyle::Style::new().dimmed();
                                format!("{dimmed}  {change}{dimmed:#}")
                            }
                        };
                        anstream::eprint!("{line}");
                    }
                }
            }
        }
    }

    exitcode
}

/// Search for hledger files in the passed directory and its subdirectories
fn gather_files_from_directory_and_subdirectories(
    root: &FilePath,
    files: &mut Vec<(FilePath, Vec<u8>)>,
) -> Result<(), ()> {
    let mut error = false;

    let journal = std::ffi::OsStr::new("journal");
    let hledger = std::ffi::OsStr::new("hledger");
    let j = std::ffi::OsStr::new("j");

    match std::fs::read_dir(root) {
        Ok(read_dir_result) => {
            for maybe_entry in read_dir_result {
                match maybe_entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_dir() {
                            if gather_files_from_directory_and_subdirectories(
                                &FilePath::from(path),
                                files,
                            )
                            .is_err()
                            {
                                error = true;
                            }
                        } else if path.is_file() {
                            let ext = path.extension();
                            if let Some(ext) = ext {
                                if [journal, hledger, j].contains(&ext) {
                                    let file_path: FilePath = path.into();
                                    let maybe_file_content = read_file(&file_path);
                                    if let Ok(content) = maybe_file_content {
                                        files.push((file_path, content));
                                    } else {
                                        error = true;
                                    }
                                }
                            }
                        } else if path.is_symlink() {
                            eprintln!(
                                "Path {path:?} is a symlink. Symbolic links are not supported."
                            );
                            error = true;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading directory {root}: {e}");
                        error = true;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading directory {root}: {e}");
            error = true;
        }
    }

    if error {
        Err(())
    } else {
        Ok(())
    }
}

fn read_file(file_path: &FilePath) -> Result<Vec<u8>, ()> {
    std::fs::read(file_path).map_err(|e| {
        eprintln!("Error reading file {file_path}: {e}");
    })
}

fn read_stdin() -> Vec<u8> {
    let mut buffer = Vec::new();
    _ = std::io::stdin().read_to_end(&mut buffer).map_err(|e| {
        eprintln!("Error reading from STDIN: {e}");
    });
    buffer
}
