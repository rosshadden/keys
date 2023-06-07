use clap::{Parser, Subcommand};
use keys::Keys;
use tokio::io;

mod keys;

static TCP_ADDRESS: &str = "127.0.0.1:1234";

#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
struct Cli {
	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
	Get {},
	Set { layer: String },
	Toggle { layers: Vec<String> },
	Watch {},
}

#[tokio::main]
async fn main() -> io::Result<()> {
	let cli = Cli::parse();
	let keys = Keys::new(TCP_ADDRESS);

	let result = match cli.command.unwrap() {
		Commands::Get {} => {
			Some(keys.get().await?)
		},
		Commands::Set { layer } => {
			Some(keys.set(layer).await?)
		},
		Commands::Toggle { layers } => {
			Some(keys.toggle(layers).await?)
		},
		Commands::Watch {} => {
			keys.watch().await?;
			None
		},
	};

	if let Some(result) = result {
		println!("{}", result);
	}

	Ok(())
}
