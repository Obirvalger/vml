VML allows using of two types of networks:
1. user network;
2. tap network.

For using user network just set in vm config or in default section of main
config `net.type = "user"`.

Tap network is more complicated. It needs all tap devices to be created and
network configured manually. It will be explained via following example.

First of all create tap devices and network. It can be done via `vnet.sh`
script running from root.

Then create vms.
```
vml create -n net/fst net/snd
```

Copy `vml-common.toml` file to `<vms-dir>/net` directory.

Place the following content in fst and snd vms configs.

```
$ cat net/fst/vml.toml
net.type = "tap"
net.address = "172.16.0.2/24"
net.tap = "tap0"
```

```
$ cat net/snd/vml.toml
net.type = "tap"
net.address = "172.16.0.3/24"
net.tap = "tap1"
```

Finally run vms.
```
vml start --wait-ssh -p net
```

In contrast with user network, in tap network ping works.
