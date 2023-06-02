use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	Get {},
	Set { layer: String },
	Start {},
	Stop {},
}

fn start() {
	print!("start");
}

fn stop() {
	print!("stop");
}

fn get() {
	print!("get");
}

fn set(layer: String) {
	print!("set: {}", layer);
}

fn main() {
	let cli = Cli::parse();

	match cli.command {
		Commands::Get {} => get(),
		Commands::Set { layer } => set(layer),
		Commands::Start {} => start(),
		Commands::Stop {} => stop(),
	}
}
