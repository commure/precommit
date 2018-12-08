use clap::ArgMatches;
use colored::*;
use failure::Error;
use git2::{Repository, StatusEntry, StatusOptions, Statuses};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Error as IOError, Write};
use std::process::Command;

#[derive(Fail, Debug)]
#[fail(display = "Hooks failed: {}", _0)]
struct HookError(String);

#[derive(Fail, Debug)]
#[fail(display = "Hook '{}' failed with error '{}'", _0, _1)]
struct CommandError(String, IOError);

#[derive(Fail, Debug)]
#[fail(display = "Invalid hook yaml file '{}'", _0)]
struct YamlSerializeError(String);

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

fn load_hooks(matches: &ArgMatches) -> Result<Hooks, Error> {
  let hooks_file_path = matches.values_of("hooks").unwrap().next().unwrap();
  let mut hooks_file = String::new();
  File::open(hooks_file_path)?.read_to_string(&mut hooks_file)?;

  Ok(
    serde_yaml::from_str(&hooks_file)
      .map_err(|_| YamlSerializeError(hooks_file_path.to_string()))?,
  )
}

fn get_staged_files(repo: &Repository) -> Result<Statuses, Error> {
  let mut status_options = StatusOptions::new();
  status_options.include_ignored(false);
  status_options.include_unmodified(false);

  let statuses = repo.statuses(Some(&mut status_options))?;
  Ok(statuses)
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

pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
  let skip_hooks: HashSet<_> = matches
    .values_of("skip")
    .map(|v| v.collect())
    .unwrap_or_else(HashSet::new);

  let hook_type = matches.values_of("hook_type").unwrap().next().unwrap();
  let hook_config = load_hooks(matches)?;

  let repo = Repository::init("./")?;

  // SKIP Merge Commits.
  if let Ok(_v) = repo.revparse("MERGE_HEAD") {
    println!(
      "{}",
      "Skipping precommit because this is a merge commit".blue()
    );
    return Ok(());
  }

  let statuses = get_staged_files(&repo)?;
  let mut hooks_failed = vec![];

  if let Some(hooks) = hook_config.get(hook_type) {
    for hook_name in hooks.keys().filter(|h| skip_hooks.get::<str>(h).is_none()) {
      let hook = &hooks[hook_name];
      let regex = Regex::new(&hook.regex).unwrap();
      let mut hook_failed = false;
      let mut hook_ran = false;
      for entry in statuses
        .iter()
        .filter(|e| !(e.status().is_wt_deleted() || e.status().is_index_deleted()))
      {
        let file_path = entry.path().unwrap();
        if regex.is_match(file_path) {
          hook_ran = true;
          for command in &hook.commands {
            let output = create_command(&command, &entry)
              .output()
              .map_err(|e| CommandError(hook_name.clone(), e))?;
            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;

            hook_failed = hook_failed || !output.status.success();
          }
        }
      }

      if hook_ran {
        print_hook_output(hook_name, hook_failed);
      }
      if hook_failed {
        hooks_failed.push(hook_name.clone());
      }
    }
  }

  if !hooks_failed.is_empty() {
    Err(HookError(hooks_failed.join(",")))?
  } else {
    Ok(())
  }
}
