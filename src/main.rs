extern crate git2;
extern crate yaml_rust;
#[macro_use]
extern crate clap;
use clap::App;

mod compile;
mod run;

fn run() -> Result<(), ()> {
  let cli_config = load_yaml!("cli.yml");
  let matches = App::from_yaml(cli_config).get_matches();
  match matches.subcommand() {
    ("compile", Some(matches)) => compile::execute(matches),
    ("run", Some(matches)) => run::execute(matches),
    _ => unreachable!(),
  }
}

fn main() {
  if let Err(()) = run() {
    println!("Exit during compilation");
    std::process::exit(1);
  }
}
