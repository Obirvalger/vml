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
cp config.toml ~/.config/vml
```
And edit config with your preferences.

## Run
Suppose you set vms-dir to `/home/user/vms/vml` and creating vm named `test`,
create the directory and set it current working directory.
```
mkdir -p /home/user/vms/vml/test
cd /home/user/vms/vml/test
```

Then download the image. Assume using image with cloud-init, .e.g
http://ftp.altlinux.org/pub/distributions/ALTLinux/images/Sisyphus/cloud/alt-sisyphus-cloud-x86_64.qcow2
however you could use any image with ssh access configured, but without using
cloud-init.
```
wget http://ftp.altlinux.org/pub/distributions/ALTLinux/images/Sisyphus/cloud/alt-sisyphus-cloud-x86_64.qcow2
```

Rename image with respect to the name of vm - test.qcow in out example.
```
mv alt-sisyphus-cloud-x86_64.qcow2 test.qcow2
```

Create `vml.toml` file with vm configuration, all fields are default, so colud
be used just empty file!
```
touch vml.toml
```

Some fields of the `vml.toml` have names as `default` section fields of the
`config.toml` file.

Finally start the vm and ssh. Option `-c` used to mount cloud-init data.
```
vml start test -c
vml ssh test
```
