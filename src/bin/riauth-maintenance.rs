use clap::Parser;

#[tokio::main]
async fn main() {
    let json = std::env::args().any(|arg| arg == "--json");
    let cli = match riauth::cli::local::MaintenanceCli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            if !error.use_stderr() {
                error.exit();
            }
            if json {
                println!(
                    "{}",
                    serde_json::json!({"schema_version": "riauth.cli/v1", "ok": false, "error": {"code": "usage", "message": error.to_string(), "retryable": false}, "exit_code": 2})
                );
            }
            let _ = error.print();
            std::process::exit(2);
        }
    };
    if let Err(error) = riauth::cli::local::run(cli).await {
        std::process::exit(riauth::cli::report_error(&error, json));
    }
}
