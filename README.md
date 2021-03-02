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

Copy config.toml to user config directory.
```
mkdir -p ~/.config/vml
cp files/config.toml ~/.config/vml
```
And edit config with your preferences.

Copy images file.
```
mkdir -p ~/.local/share/vml/images
cp files/images.toml ~/.local/share/vml/images
```

## Run
Suppose you set vms-dir to `/home/user/vms/vml` and creating vm named `test`,
using alt-sisyphus image.
```
mkdir -p /home/user/vms/vml/test
vml images pull alt-sisyphus
vml create -i alt-sisyphus -n test
```

Some fields of the `vml.toml` have names as `default` section fields of the
`config.toml` file.

Finally start the vm with `test` name and ssh. Option `-c` used to mount
cloud-init data.
```
vml start -n test -c
vml ssh -n test
```
