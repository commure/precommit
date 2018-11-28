use clap::ArgMatches;
use colored::*;
use git2::{Repository, StatusEntry, StatusOptions, Statuses};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Write};
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

  serde_yaml::from_str(&hooks_file).unwrap_or_else(|_| {
    panic!(
      "{}: {}",
      "failed to deserialize hooks yaml file".red(),
      hooks_file_path
    )
  })
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

fn print_hook_output(hook_name: &str, hook_failed: bool) {
  if !hook_failed {
    println!("{} {} :  {}", "✓".green(), hook_name, "passed".green());
  } else {
    println!("{} {} : {}", "✗".red(), hook_name, "failed".red());
  }
}

pub fn execute(matches: &ArgMatches) -> Result<(), ()> {
  let skip_hooks: HashSet<_> = matches
    .values_of("skip")
    .map(|v| v.collect())
    .unwrap_or_else(HashSet::new);

  let hook_type = matches.values_of("hook_type").unwrap().next().unwrap();
  let hook_config = load_hooks(matches);

  let repo = Repository::init("./").expect("failed to find git repo");
  let statuses = get_staged_files(&repo);
  let mut err = false;

  if let Some(hooks) = hook_config.get(hook_type) {
    for hook_name in hooks.keys().filter(|h| skip_hooks.get::<str>(h).is_none()) {
      let hook = &hooks[hook_name];
      let regex = Regex::new(&hook.regex).unwrap();
      let mut hook_failed = false;
      let mut hook_ran = false;
      for entry in statuses.iter() {
        let file_path = entry.path().unwrap();
        if regex.is_match(file_path) {
          hook_ran = true;
          for command in &hook.commands {
            let output = create_command(&command, &entry)
              .output()
              .unwrap_or_else(|_| {
                panic!(
                  "{}: {}",
                  "failed to execute process for hook".red(),
                  command.command
                )
              });

            io::stdout()
              .write_all(&output.stdout)
              .expect("failed to write to stdout");
            io::stderr()
              .write_all(&output.stderr)
              .expect("failed to write to stderr");

            hook_failed = hook_failed || !output.status.success();
          }
        }
      }

      if hook_ran {
        print_hook_output(hook_name, hook_failed);
      }
      err = err || hook_failed;
    }
  }

  if err {
    Err(())
  } else {
    Ok(())
  }
}
