# cloud-init

Cloud-init is the method for cloud instance initialization.

## Metadata

VML uses nocloud data source. That is providing to vm additional drive with cloud-init metadata.
This metadata could be created it from user-data config file via `cloud-localds` utility. Next
command creates `my-seed.img` metadata from `user-data.yaml` config.
```
$ cloud-localds my-seed.img user-data.yaml
```

## Configs

All SSH_PUBLIC_KEY entries in configs should be replaced with your ssh public key.

Examples of cloud configs - https://cloudinit.readthedocs.io/en/latest/topics/examples.html.

There are some minimal examples of config file:
* `root-key-user-data.yaml` - sets only root ssh key.
* `user-user-data.yaml` - adds `user` user in `wheel` group and sets `user` and root ssh keys.
