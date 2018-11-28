use clap::ArgMatches;
use git2::Repository;
use std::env::current_exe;
use std::fs::{set_permissions, File, Permissions};
use std::io::prelude::*;
use std::os::unix::prelude::PermissionsExt;

pub fn script_gen(
  location_command: &str,
  hook: &str,
  hook_config_file: &str,
  skip_str: &str,
) -> String {
  format!(
    "
#!/bin/sh
# {}
# Hook created by precommit
{} run {} {} {}
",
    hook, location_command, hook, hook_config_file, skip_str
  )
}

pub fn execute(matches: &ArgMatches) -> Result<(), ()> {
  // let location = "./target/debug/precommit";
  let skip_hooks: Vec<_> = matches
    .values_of("skip")
    .map(|v| v.collect())
    .unwrap_or_else(|| vec![]);
  let executable_path = current_exe().expect("could not get precommit location");
  let location = executable_path.to_string_lossy();
  Repository::init("./").expect("precommit lib must be at root of git repo!");

  let hook = "pre-commit";
  let hook_config_file = matches
    .values_of("hook_config_file")
    .unwrap()
    .next()
    .unwrap();
  let skip_str = if skip_hooks.is_empty() {
    "".to_string()
  } else {
    format!("-s={}", skip_hooks.join(","))
  };

  let template = script_gen(&location, hook, hook_config_file, &skip_str);
  let git_hook_file = format!(".git/hooks/{}", hook);

  let mut file = File::create(&git_hook_file).expect("failed to create file");
  file
    .write_all(&template.into_bytes())
    .expect("failed to write to file");

  set_permissions(&git_hook_file, Permissions::from_mode(0o777)).expect("Failed to chmod: {}");

  Ok(())
}
