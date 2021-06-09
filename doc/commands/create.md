# create
* Names are specified via `-n/--names` flag. If used only one name flag can be
  omitted.

  * Create one vm from the default (set in main vml config) image.
```
vml create -n test
```
  or
```
vml create test
```
  * Create multiple vms from the default image.
```
vml create -n test1 test2 test3
```

* Image is specified via `-i/--image` flag.
  * Create one vm using specified (e.g. alt-p9) image.
```
vml create -i alt-p9 test-p9
```

* To specify behavior of creating already exists vm use one of `--exists-*`
  flags.
  * Replace existing vms disk with one from default image.
```
vml create test --exists-replace
```

* Other flags allow specifying vms parameters.
  * Create vm from default image with 4 gigabytes of ram and resizing disk to
    64 gigabytes.
```
vml create -n test-big --memory 4G --minimum-disk-size 64G
```
