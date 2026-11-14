#[cfg(windows)]
mod app;
#[cfg(windows)]
mod release;
#[cfg(windows)]
fn main() -> std::process::ExitCode {
    match app::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
#[cfg(not(windows))]
fn main() {
    eprintln!("InstanceDeck currently supports Windows 10 and 11 only.");
    std::process::exit(1);
}
