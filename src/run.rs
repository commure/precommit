use clap::ArgMatches;
use git2::{Repository, StatusEntry, StatusOptions, StatusShow, Statuses};
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Write};
use std::process::Command;
use yaml_rust::{Yaml, YamlLoader};

fn load_hooks(matches: &ArgMatches) -> std::vec::Vec<Yaml> {
  let hooks_file_path = matches.values_of("hooks").unwrap().next().unwrap();
  let mut hooks_file = String::new();
  File::open(hooks_file_path)
    .expect("file not found")
    .read_to_string(&mut hooks_file)
    .expect("could not create string");

  YamlLoader::load_from_str(&hooks_file).expect("failed to read hooks")
}

fn get_staged_files<'a>(repo: &'a Repository) -> Statuses<'a> {
  let mut status_options = StatusOptions::new();
  status_options.show(StatusShow::Index);
  status_options.include_ignored(false);
  status_options.include_unmodified(false);

  repo
    .statuses(Some(&mut status_options))
    .expect("error getting statuses")
}

fn build_language_hash<'a>(hook: &'a Yaml) -> Option<HashSet<&'a Yaml>> {
  match &hook["languages"] {
    Yaml::Array(array) => Some(array.iter().collect()),
    _ => None,
  }
}

fn yaml_to_string(yaml: &Yaml) -> Option<String> {
  match yaml {
    Yaml::String(string) => Some(string.clone()),
    _ => None,
  }
}

fn yaml_to_array<'a>(yaml: &'a Yaml) -> Option<&'a Vec<Yaml>> {
  match yaml {
    Yaml::Array(array) => Some(array),
    _ => None,
  }
}

fn yaml_to_bool(yaml: &Yaml) -> bool {
  match yaml {
    Yaml::Boolean(boolean) => *boolean,
    _ => false,
  }
}

fn create_command(hook: &Yaml, entry: &StatusEntry) -> Command {
  let command_str = yaml_to_string(&hook["command"]).unwrap();
  let mut command = Command::new(command_str);
  let default = vec![];
  let args = yaml_to_array(&hook["arguments"]).unwrap_or(&default);
  for arg in args {
    let arg_string = yaml_to_string(arg).unwrap();
    if arg_string == "<filename>".to_string() {
      command.arg(entry.path().unwrap());
    } else {
      command.arg(arg_string);
    }
  }

  command
}

pub fn execute(matches: &ArgMatches) -> Result<(), ()> {
  let hook_type = matches.values_of("hook_type").unwrap().next().unwrap();
  let hook_config = load_hooks(matches);

  let repo = Repository::init("./").expect("failed to find git repo");
  let statuses = get_staged_files(&repo);
  let hooks = yaml_to_array(&hook_config[0][hook_type]).unwrap();

  for hook in hooks {
    let language_hash = build_language_hash(&hook).unwrap();
    for entry in statuses.iter() {
      let chunks = entry.path().unwrap().split('.').collect::<Vec<&str>>();
      if let Some(_v) = language_hash.get(&Yaml::String(chunks[chunks.len() - 1].to_string())) {
        let output = create_command(&hook, &entry)
          .output()
          .expect("failed to execute process");
        io::stdout()
          .write(&output.stdout)
          .expect("failed to write to stdout");
        io::stderr()
          .write(&output.stderr)
          .expect("failed to write to stderr");
      }
    }
  }

  Ok(())
}
