use clap::Parser;
use sc_chain_spec::ChainSpec;
use std::collections::HashMap;

mod spec;

use spec::live;
use spec::local;
use spec::KreivoChainSpec;

#[derive(Parser)]
struct Cli {
	/// The chain spec to generate.
	chain: String,

	/// Generate the chain spec as raw?
	#[arg(long)]
	raw: bool,
}

fn main() -> Result<(), String> {
	let cli = Cli::parse();

	let supported_chains = HashMap::<_, Box<dyn Fn() -> Result<Box<dyn ChainSpec>, String>>>::from([
		("kreivo", Box::new(live::chain_spec) as Box<_>),
		("live", Box::new(live::chain_spec) as Box<_>),
		("kreivo-local", Box::new(local::chain_spec) as Box<_>),
		("local", Box::new(local::chain_spec) as Box<_>),
	]);

	if let Some(function) = supported_chains.get(&*cli.chain) {
		let chain_spec = (*function)()?.as_json(cli.raw)?;
		print!("{chain_spec}");
		Ok(())
	} else {
		let supported = supported_chains.keys().enumerate().fold(String::new(), |c, (n, k)| {
			let extra = if n + 1 < supported_chains.len() { ", " } else { "" };
			format!("{c}{k}{extra}")
		});
		if cli.chain.ends_with(".json") {
			let chain_spec = Box::new(KreivoChainSpec::from_json_file(cli.chain.into())?.as_json(cli.raw)?);
			print!("{chain_spec}");
			Ok(())
		} else {
			Err(format!("Unknown chain, only supported: {supported} or a json file"))
		}
	}
}
