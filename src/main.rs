use clap::{Parser, Subcommand};
use futures_util::pin_mut;
use keys::Keys;
use serde::{Deserialize,Serialize};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;

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
	let keys = Keys {};
	let stream = TcpStream::connect(TCP_ADDRESS).await?;
	let s = keys.read(stream);
	pin_mut!(s);

	while let Some(Ok(layer)) = s.next().await {
		return Ok(layer);
	}

	Ok(String::new())
}

async fn set(layer: String) -> io::Result<String> {
	let mut tcp_stream = TcpStream::connect(TCP_ADDRESS).await?;

	let payload = Payload::ChangeLayer { layer: layer.clone() };
	if let Ok(json) = serde_json::to_string(&payload) {
		tcp_stream.write_all(json.as_bytes()).await.expect("failed to send data");
	}

	Ok(layer)
}

async fn watch() -> io::Result<()> {
	let keys = Keys {};
	let stream = TcpStream::connect(TCP_ADDRESS).await?;
	let s = keys.read(stream);
	pin_mut!(s);

	while let Some(Ok(layer)) = s.next().await {
		println!("{}", layer);
	}

	Ok(())
}

async fn toggle(layers: Vec<String>) -> io::Result<String> {
	println!("{:?}", layers);
	Ok(String::new())
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
