use clap::{Parser, Subcommand};
use vyom::{repl, server};

const CHUNK_SIZE: usize = 64 * 1024;

#[derive(Debug, Parser)]
struct Args {
    /// Directory to store data
    #[arg(short, long, default_value = "./data")]
    data_dir: String,
    /// Port to bind the server to
    #[arg(short, long, default_value = "8966")]
    port: u16,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start a REPL session
    Repl,
    /// Serve on HTTP
    Serve,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let cli = Args::parse();
    tracing::debug!("Args: {:?}", &cli);
    let root_dir = cli.data_dir;
    let port = cli.port;

    match cli.command {
        Some(Command::Repl) => {
            repl::start_repl(&root_dir, CHUNK_SIZE).await?;
        }
        Some(Command::Serve) => {
            server::start_server(&root_dir, CHUNK_SIZE, port).await?;
        }
        None => {
            println!("No command provided. Use --help for usage information.");
            println!("Available commands:");
            println!("  repl   - Start a REPL session");
            println!("  serve  - Start HTTP server");
        }
    }
    Ok(())
}
