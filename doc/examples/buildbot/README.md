# Example of buildbot setup using ansible

Example setup of buildbot with one master vm and one worker vm.

Assume that network was set up as of [network documentation example](../../network/vnet.sh) and
run vms.
```
vml run --net-tap tap0 --net-address 172.16.0.2/24 --net-gateway 172.16.0.1 ans/bb/master
vml run ans/bb/worker
```

After vms successfully created them can be provisioned. Run ansible with
[dynamic inventory](../../../files/scripts/inventory.py) and [playbook](playbook.yaml).
```
ansible-playbook -i ../../../files/scripts/inventory.py playbook.yaml -e task_number=
```
Via `task_number` variable could be set task to use for buildbot packages instead of main the repo.
