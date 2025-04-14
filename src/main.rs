use hledger_fmt::cli;

fn main() {
    let exitcode = cli::run(cli::build());
    std::process::exit(exitcode);
}
