use std::io::Write;

use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

use crate::FileStorage;

pub struct ReplSession {
    storage: FileStorage,
}

impl ReplSession {
    pub async fn new(
        root_dir: &str,
        chunk_size: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Initializing storage at: {root_dir}");
        let storage = FileStorage::new(root_dir, chunk_size).await?;
        Ok(Self { storage })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        Self::print_welcome();
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);

        loop {
            Self::print_prompt()?;
            let Ok(input) = self.read_input(&mut reader).await else {
                eprintln!("Failed to read input");
                continue;
            };

            if Self::should_exit(&input) {
                println!("Goodbye!");
                break Ok(());
            }

            if let Err(e) = self.handle_command(&input).await {
                eprintln!("Error: {e}");
            }
        }
    }

    fn print_welcome() {
        println!("vyom REPL started.");
        println!("Available commands:");
        println!("  put <file> <path>  - Store a file from the given path");
        println!("  get <file>         - Retrieve and display a file");
        println!("  del <file>         - Delete a file");
        println!("  all                - List all stored files");
        println!("  help               - Show this help message");
        println!("  exit, quit         - Exit the REPL");
        println!();
    }

    fn print_prompt() -> Result<(), std::io::Error> {
        print!("> ");
        std::io::stdout().flush()
    }

    async fn read_input(
        &self,
        reader: &mut BufReader<tokio::io::Stdin>,
    ) -> Result<String, std::io::Error> {
        let mut input = String::new();
        reader.read_line(&mut input).await?;
        Ok(input.trim().to_string())
    }

    fn should_exit(input: &str) -> bool {
        matches!(input, "exit" | "quit")
    }

    async fn handle_command(&self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<&str> = input.split_whitespace().collect();
        if args.is_empty() {
            return Ok(());
        }

        match args[0] {
            "put" => self.handle_put(&args).await,
            "get" => self.handle_get(&args).await,
            "del" => {
                self.handle_del(&args);
                Ok(())
            }
            "all" => {
                self.handle_all();
                Ok(())
            }
            "help" => {
                Self::print_help();
                Ok(())
            }
            "" => Ok(()),
            _ => {
                eprintln!(
                    "Unknown command: '{}'. Type 'help' for available commands.",
                    args[0]
                );
                Ok(())
            }
        }
    }

    async fn handle_put(&self, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        if args.len() != 3 {
            eprintln!("Usage: put <file> <path>");
            return Ok(());
        }

        let file_name = args[1];
        let file_path = args[2];

        println!("Storing file '{file_name}' from path '{file_path}'...");

        match File::open(file_path).await {
            Ok(reader) => {
                self.storage.put_file(file_name, reader, None).await?;
                println!("Successfully stored file '{file_name}'");
            }
            Err(e) => {
                eprintln!("Failed to open file '{file_path}': {e}");
            }
        }

        Ok(())
    }

    async fn handle_get(&self, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        if args.len() != 2 {
            eprintln!("Usage: get <file>");
            return Ok(());
        }

        let file_name = args[1];
        println!("Retrieving file '{file_name}'...");

        match self.storage.get_file(file_name).await? {
            Some((data, metadata)) => {
                println!("File content ({} bytes):", data.len());
                if let Some(mime_type) = metadata.mime_type() {
                    println!("MIME type: {mime_type}");
                }
                println!("{}", String::from_utf8_lossy(&data));
            }
            None => {
                println!("File '{file_name}' not found");
            }
        }

        Ok(())
    }

    fn handle_del(&self, args: &[&str]) {
        if args.len() != 2 {
            eprintln!("Usage: del <file>");
            return;
        }

        let file_name = args[1];
        println!("Deleting file '{file_name}'...");

        match self.storage.del_file(file_name) {
            Ok(()) => println!("Successfully deleted file '{file_name}'"),
            Err(e) => eprintln!("Failed to delete file '{file_name}': {e}"),
        }
    }

    fn handle_all(&self) {
        println!("Listing all stored files:");

        match self.storage.all_files() {
            Ok(files) => {
                if files.is_empty() {
                    println!("No files stored");
                } else {
                    for (i, file) in files.iter().enumerate() {
                        println!("  {}. {}", i + 1, file);
                    }
                    println!("Total: {} file(s)", files.len());
                }
            }
            Err(e) => {
                eprintln!("Error listing files: {e}");
            }
        }
    }

    fn print_help() {
        println!("Available commands:");
        println!("  put <file> <path>  - Store a file from the given path");
        println!("  get <file>         - Retrieve and display a file");
        println!("  del <file>         - Delete a file");
        println!("  all                - List all stored files");
        println!("  help               - Show this help message");
        println!("  exit, quit         - Exit the REPL");
    }
}

pub async fn start_repl(
    root_dir: &str,
    chunk_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let repl = ReplSession::new(root_dir, chunk_size).await?;
    repl.start().await
}
