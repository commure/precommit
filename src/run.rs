use clap::ArgMatches;
use git2::{Repository, StatusEntry, StatusOptions, Statuses};
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

#[derive(Deserialize, Debug, Clone)]
struct HookCommand {
  command: String,
  #[serde(default)]
  arguments: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct Hook {
  commands: Vec<HookCommand>,
  regex: String,
  #[serde(default)]
  description: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Hooks {
  #[serde(rename = "pre-commit")]
  pre_commit: Option<HashMap<String, Hook>>,
}

impl Hooks {
  fn get(&self, hook_type: &str) -> Option<HashMap<String, Hook>> {
    match hook_type {
      "pre-commit" => self.pre_commit.clone(),
      _ => {
        println!("Unimplemented hook type of {}", hook_type);
        None
      }
    }
  }
}

fn load_hooks(matches: &ArgMatches) -> Hooks {
  let hooks_file_path = matches.values_of("hooks").unwrap().next().unwrap();
  let mut hooks_file = String::new();
  File::open(hooks_file_path)
    .expect("file not found")
    .read_to_string(&mut hooks_file)
    .expect("could not create string");

  serde_yaml::from_str(&hooks_file).unwrap()
}

fn get_staged_files(repo: &Repository) -> Statuses {
  let mut status_options = StatusOptions::new();
  status_options.include_ignored(false);
  status_options.include_unmodified(false);

  repo
    .statuses(Some(&mut status_options))
    .expect("error getting statuses")
}

fn create_command(h_command: &HookCommand, entry: &StatusEntry) -> Command {
  let mut command = Command::new(&h_command.command);
  for arg in &h_command.arguments {
    if arg == "<filename>" {
      command.arg(entry.path().unwrap());
    } else {
      command.arg(arg);
    }
  }

  command
}

pub fn execute(matches: &ArgMatches) -> Result<(), ()> {
  let hook_type = matches.values_of("hook_type").unwrap().next().unwrap();
  let hook_config = load_hooks(matches);

  let repo = Repository::init("./").expect("failed to find git repo");
  let mut repo_index = repo.index().expect("failed to get repo index");
  let statuses = get_staged_files(&repo);
  let mut err = false;

  if let Some(hooks) = hook_config.get(hook_type) {
    for key in hooks.keys() {
      let hook = &hooks[key];
      let regex = Regex::new(&hook.regex).unwrap();
      for entry in statuses.iter() {
        let file_path = entry.path().unwrap();
        if regex.is_match(file_path) {
          for command in &hook.commands {
            let output = create_command(&command, &entry)
              .output()
              .unwrap_or_else(|_| panic!("failed to execute process hook {}", command.command));

            io::stdout()
              .write_all(&output.stdout)
              .expect("failed to write to stdout");
            io::stderr()
              .write_all(&output.stderr)
              .expect("failed to write to stderr");

            if !output.status.success() {
              err = true;
            }
          }
        }
      }
    }
  }

  if err {
    Err(())
  } else {
    Ok(())
  }
}
