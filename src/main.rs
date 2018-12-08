#[macro_use]
extern crate clap;
extern crate colored;
#[macro_use]
extern crate failure;
extern crate git2;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
use clap::App;
use colored::*;
use failure::Error;

mod compile;
mod run;

fn run() -> Result<(), Error> {
  let cli_config = load_yaml!("cli.yml");
  let matches = App::from_yaml(cli_config).get_matches();
  match matches.subcommand() {
    ("install", Some(matches)) => compile::execute(matches),
    ("run", Some(matches)) => run::execute(matches),
    _ => unreachable!(),
  }
}

fn main() {
  if let Err(e) = run() {
    println!("{}", e.to_string().red());
    std::process::exit(1);
  }
}
