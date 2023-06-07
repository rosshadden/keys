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

async fn get() -> io::Result<String> {
	let keys = Keys::new(TCP_ADDRESS);
	keys.get().await
}

async fn set(layer: String) -> io::Result<String> {
	let keys = Keys::new(TCP_ADDRESS);
	keys.set(layer).await
}

async fn watch() -> io::Result<()> {
	let keys = Keys::new(TCP_ADDRESS);
	keys.watch().await
}

async fn toggle(layers: Vec<String>) -> io::Result<String> {
	let keys = Keys::new(TCP_ADDRESS);
	keys.toggle(layers).await
}

#[tokio::main]
async fn main() -> io::Result<()> {
	let cli = Cli::parse();

	match cli.command.unwrap() {
		Commands::Get {} => {
			println!("{}", get().await?);
		},
		Commands::Set { layer } => {
			println!("{}", set(layer).await?);
		},
		Commands::Toggle { layers } => {
			println!("{}", toggle(layers).await?);
		},
		Commands::Watch {} => {
			watch().await?;
		},
	}

	Ok(())
}
