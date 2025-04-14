use clap::{value_parser, Arg, ArgAction, Command};

#[doc(hidden)]
/// Build the hledger-fmt CLI with clap.
pub fn build() -> Command {
    Command::new("hledger-fmt")
        .long_about("An opinionated hledger's journal files formatter.")
        .override_usage("hledger-fmt [OPTIONS] [FILES]...\n")
        .arg(
            Arg::new("files")
                .help(
                    "Paths of files to format. To read from STDIN pass '-'.\n\
            \n\
            If not defined, hledger-fmt will search for hledger files in the \
            current directory and its subdirectories (those that have the \
            extensions '.journal', '.hledger' or '.j'). \
            If the paths passed are directories, hledger-fmt will search for \
            hledger files in those directories and their subdirectories.",
                )
                .action(ArgAction::Append)
                .value_parser(value_parser!(String))
                .value_name("FILES")
                .num_args(1..),
        )
        .arg(
            Arg::new("fix")
                .long("fix")
                .help(
                    "Fix the files in place. WARNING: this is a potentially destructive \
           operation, make sure to make a backup of your files or print the diff \
           first. If not passed, hledger-fmt will print the diff between the \
           original and the formatted file.",
                )
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-diff")
                .long("no-diff")
                .help(
                    "Don't print diff between original and formatted files, \
           but formatted content instead.",
                )
                .action(ArgAction::SetTrue),
        )
        .disable_help_flag(true)
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .help("Print help.")
                .action(ArgAction::Help),
        )
        .disable_version_flag(true)
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .help("Print version.")
                .action(ArgAction::Version),
        )
        .after_help("To disable colors in the output, use the environment variable NO_COLOR.")
}
