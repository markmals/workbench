use workbench::failure::{ErrorCode, Failure};

fn main() {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            let failure =
                Failure::precondition(format!("cannot read the current directory: {error}"));
            eprintln!("{}", failure.to_json());
            std::process::exit(ErrorCode::PreconditionFailed.exit_status());
        }
    };
    std::process::exit(workbench::app::run(std::env::args_os().collect(), cwd));
}
