# GHooks

## Installation
To install binary : 

`cargo install --git=http://github.com/commure/ghooks.git`

Then in repository

`precommit compile <hook_yaml_file>`

## Information
Currenlty this only runs git pre-commit hook.

## Config file
```yaml
<hook_name>:
  commands: <list of commands to execute for given hook>
    - command: <command to run>
      arguments: <command_arguments note special variable of filename which will fill in file that will be run>
  regex: <regex to test whether or not to run a given file>
  restage: <optional boolean saying whether to restage file in git after processing command>
```

## Example

```yaml
pre-commit:
  prettier: 
    commands:
      - command: prettier
        arguments: [--write, <filename>]
    regex: '.*(js|jsx|scss|md)$'
    restage: true

  eslint: 
    commands:
      - command: eslint
        arguments: [<filename>]
    regex: '.*(js|sass|css)$'
    restage: true

  rustfmt: 
    commands:
      - command: rustfmt
        arguments: [<filename>]
    regex: '.*(rs)$'
    restage: true

  clippy: 
    commands:
      - command: cargo
        arguments: [clippy]
    regex: '.*(rs)$'

  print_yaml:
    commands:
      - command: cat
        arguments: [<filename>]
    regex: '.*(yaml|yml)$'

```
