# vml
VML is a tool for easily and transparently work with qemu virtual machines.
Virtaul machines presend as directories with vml.toml files in it.

## Build and setup
All needed dependencies seved into vendor directory, so it can be built
in the offline mode.
```
cargo build --release --offline
```

Then copy created executable to appropriate path, e.g. ~/bin/vml, if
~/bin is in your PATH.
```
cp target/release/vml ~/bin/vml
```

## Run
All needed files are copied with any command. For example list available to
pull images.
```
vml images available
```

Or get completion for your shell (zsh in example).
```
vml completion zsh
```

Then edit config with your preferences.
```
$EDITOR ~/.config/vml/config.toml
```

About cloud-init image see [docs](doc/cloud-init).

To create vm named `test`, using `alt-sisyphus` image.
```
vml images pull alt-sisyphus
vml create -i alt-sisyphus -n test
```

VM test is described via directory `test` in `<vms_dir>` (vms_dir from config)
and within files: `test.qcow` is a disk image, `vml.tml` is a current vm config
file. By default `vml.toml` is empty, but it is needed to mark the directory as
`vml` vm. Some fields of the `vml.toml` have names as `default` section fields
of the main config file `~/.config/vml/config.toml`.

Finally start the vm with `test` name and ssh. Option `-c` used to mount
cloud-init data.
```
vml start -c -n test
vml ssh -n test
```
