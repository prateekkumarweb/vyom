use std::io::Write;

use axum::{Router, routing::get};
use clap::{Parser, Subcommand};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use vyom::FileStorage;

const CHUNK_SIZE: usize = 64 * 1024;

#[derive(Debug, Parser)]
struct Args {
    /// Directory to store data
    #[arg(short, long, default_value = "./data")]
    data_dir: String,
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
    let cli = Args::parse();
    let root_dir = cli.data_dir;

    match cli.command {
        Some(Command::Repl) => {
            start_repl(&root_dir).await?;
        }
        Some(Command::Serve) => {
            start_server(&root_dir).await?;
        }
        None => {
            println!("No command provided. Use --help for usage information.");
        }
    }
    Ok(())
}

async fn start_repl(root_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing storage at: {}", root_dir);
    let storage = FileStorage::new(root_dir, CHUNK_SIZE).await?;
    println!(
        "vyom REPL started. Enter commands: get <file>, put <file> <path>, del <file>, or 'exit' to quit."
    );
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        if reader.read_line(&mut input).await.is_err() {
            eprintln!("Failed to read input");
            continue;
        }
        let input = input.trim();
        if input == "exit" || input == "quit" {
            break Ok(());
        }
        let args: Vec<&str> = input.split_whitespace().collect();
        if args.is_empty() {
            continue;
        }
        match args[0] {
            "put" => {
                if args.len() != 3 {
                    eprintln!("Usage: put <file> <path>");
                    continue;
                }
                let file = args[1];
                let path = args[2];
                println!("Put: file = {file}, path = {path}");
                let reader = File::open(path).await?;
                storage.put_file(file, reader).await?;
            }
            "get" => {
                if args.len() != 2 {
                    eprintln!("Usage: get <file>");
                    continue;
                }
                let file = args[1];
                println!("Get: file = {file}");
                let data = storage.get_file(file).await?;
                if let Some(data) = data {
                    println!("File data: {}", String::from_utf8_lossy(&data));
                }
            }
            "del" => {
                if args.len() != 2 {
                    eprintln!("Usage: del <file>");
                    continue;
                }
                let file = args[1];
                println!("Del: file = {file}");
                storage.del_file(file)?;
            }
            "all" => {
                println!("Listing all files:");
                match storage.all_files() {
                    Ok(files) => {
                        for file in files {
                            println!("{file}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error listing files: {e}");
                    }
                }
            }
            _ => {
                eprintln!("Unknown command: {}", args[0]);
            }
        }
    }
}

async fn start_server(_root_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8966").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
