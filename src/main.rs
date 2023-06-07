use clap::{Parser, Subcommand};
use keys::Keys;
use serde::{Deserialize,Serialize};
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

#[derive(Debug, Deserialize, Serialize)]
enum Payload {
	LayerChange {
		#[serde(rename = "new")]
		layer: String,
	},
	ChangeLayer {
		#[serde(rename = "new")]
		layer: String,
	},
}

async fn get(keys: Keys) -> io::Result<String> {
	keys.get().await
}

async fn set(keys: Keys, layer: String) -> io::Result<String> {
	keys.set(layer).await
}

async fn watch(keys: Keys) -> io::Result<()> {
	keys.watch().await
}

async fn toggle(keys: Keys, layers: Vec<String>) -> io::Result<String> {
	keys.toggle(layers).await
}

#[tokio::main]
async fn main() -> io::Result<()> {
	let cli = Cli::parse();
	let keys = Keys::new(TCP_ADDRESS);

	match cli.command.unwrap() {
		Commands::Get {} => {
			println!("{}", get(keys).await?);
		},
		Commands::Set { layer } => {
			println!("{}", set(keys, layer).await?);
		},
		Commands::Toggle { layers } => {
			println!("{}", toggle(keys, layers).await?);
		},
		Commands::Watch {} => {
			watch(keys).await?;
		},
	}

	Ok(())
}
