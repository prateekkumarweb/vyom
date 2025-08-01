use std::io::Write;

use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use vyom::FileStorage;

const CHUNK_SIZE: usize = 64 * 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = FileStorage::new("./data", CHUNK_SIZE).await?;
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
                storage.del_file(file).await?;
            }
            "all" => {
                println!("Listing all files:");
                match storage.all_files().await {
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
