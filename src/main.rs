use hledger_fmt::cli;

fn main() {
    let exitcode = cli::run(cli::builder::build());
    std::process::exit(exitcode);
}
