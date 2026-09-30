use clap::Parser;

fn main() {
    #[cfg(feature = "platform")]
    if std::env::args().nth(1).as_deref() == Some(riauth::workflow::extension_gate::GUEST_ARGV) {
        std::process::exit(riauth::workflow::extension_gate::run_isolated_guest());
    }
    server_main();
}

#[tokio::main]
async fn server_main() {
    let json = std::env::args().any(|a| a == "--json");
    let cli = match riauth::cli::Cli::try_parse() {
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
    if let Err(error) = riauth::cli::run(cli).await {
        std::process::exit(riauth::cli::report_error(&error, json));
    }
}
