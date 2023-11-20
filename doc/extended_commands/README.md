# Extended commands

Vml subcommands could be extended via external programms (e.g. scripts) adding
them to catalogs listed in the `PATH` environment variable with names started
with vml.

## Example

1. Ensure `PATH` variable contains `~/bin`;
2. Copy `vml-xrun` to the `~/bin` catalog;
3. Now you could use `vml xrun` command that prints message before run vm:
```
    vml xrun test-ext
```
