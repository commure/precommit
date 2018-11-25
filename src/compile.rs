use clap::ArgMatches;

pub fn execute(matches: &ArgMatches) -> Result<(), ()> {
  println!("Executing compilation {:#?}", matches);
  load_yaml!("cli.yml");
  Ok(())
}
