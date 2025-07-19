use rust_singler::cli::CliApplication;
use std::process;

fn main() {
    let app = CliApplication::new();
    let args = CliApplication::parse_arguments();
    
    if let Err(e) = app.run(args) {
        eprintln!("❌ Error: {}", e);
        process::exit(1);
    }
}
