use clap::{Parser, Subcommand};
use keys::Keys;
use tokio::io;

mod keys;

#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
struct Cli {
	#[arg(short, long, default_value_t = 1234, value_parser = clap::value_parser!(u16).range(1..))]
	port: u16,

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
	let keys = Keys::new(1234);

	if cli.command.is_none() {
		Some(keys.start().await?);
		println!("{}", cli.port);
		return Ok(())
	}

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
