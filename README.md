# HOOK_EXP

## Setup
Run `cargo run compile <hooks_file>` example hooks_file would be hooks.yaml 
This will setup the git hook which is currently only a pre-commit. 

## Information
Currenlty this only runs git pre-commit hook.

## Config file
```yaml
<hook_name>:
    - command: <command to run>
      arguments: <command_arguments note special variable of filename which will fill in file that will be run>
      regex: <regex to test whether or not to run a given file>
```

## Example

```yaml
pre-commit:
    - command: eslint
      arguments:
        - <filename>
      regex: '.*(js|sass|css)$'
    - command: rustfmt
      arguments:
        - <filename>
      regex: '.*(rs)$'
    - command: prettier
      arguments:
        - --write
        - <filename>
      regex: '.*(js|jsx|scss|md)$'
    - command: cat
      arguments:
        - <filename>
      regex: '.*(yaml|yml)$'
```
