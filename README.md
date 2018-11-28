# precommit

## Installation

To install binary :

`cargo install --git=http://github.com/commure/precommit.git`

Then in repository

`precommit install <hook_yaml_file>`

## Information

Currenlty this only runs git pre-commit hook.

## Config file

```yaml
<hook_name>:
  commands: <list of commands to execute for given hook>
    - command: <command to run>
      arguments: <command_arguments note special variable of filename which will fill in file that will be run>
  regex: <regex to test whether or not to run a given file>
```

## Example

```yaml
pre-commit:
  prettier:
    commands:
      - command: prettier
        arguments: [--write, <filename>]
      - command: git
        arguments: [add, <filename>]
    regex: ".*(js|jsx|scss|md)$"

  eslint:
    commands:
      - command: eslint
        arguments: [<filename>]
      - command: git
        arguments: [add, <filename>]
    regex: ".*(js|sass|css)$"

  rustfmt:
    commands:
      - command: rustfmt
        arguments: [<filename>]
      - command: git
        arguments: [add, <filename>]
    regex: ".*(rs)$"

  clippy:
    commands:
      - command: cargo
        arguments: [clippy]
    regex: ".*(rs)$"

  print_yaml:
    commands:
      - command: cat
        arguments: [<filename>]
    regex: ".*(yaml|yml)$"
```
