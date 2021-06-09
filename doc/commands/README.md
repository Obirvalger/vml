# Commands

This directory contains examples of some main commands.

A lot of commands working with vms accept arguments to specify vms.
* `-n/--names` - whole names. This flag can be omitted if specify only one
  name.
```
vml create -n test-parents/first test-parents/second
```
* `-p/--parents` - if vm name consider as path, parent is any parenr directory
  including itself.
```
vml start -p test-parents
```
* `-t/--tags` - tags that specified in vml.toml - vm describing files.
