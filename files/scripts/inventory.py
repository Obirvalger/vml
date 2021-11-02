#!/usr/bin/python3

import ipaddress
import json
import subprocess

from collections import defaultdict
from pathlib import Path


class InventoryBuilder:
    def __init__(self):
        self.vms = json.loads(
            subprocess.run(
                ["vml", "show", "--format-json", "--all", "--running"],
                stdout=subprocess.PIPE,
            ).stdout
        )

        self.names = [vm["name"] for vm in self.vms]

        self.inventory = {
            "all": {
                "hosts": self.names,
                "vars": {"ansible_python_interpreter": "/usr/bin/python3"},
            },
            "_meta": {"hostvars": {}},
        }

    def hostvars(self):
        for vm in self.vms:
            host_vars = {}
            if host := vm.get("ssh_host"):
                host_vars["ansible_host"] = host
            if port := vm.get("ssh_port"):
                host_vars["ansible_port"] = port
            if user := vm.get("ssh_user"):
                host_vars["ansible_user"] = user
            if key := vm.get("ssh_key"):
                host_vars["ansible_ssh_private_key_file"] = key
            if options := vm.get("ssh_options"):
                host_vars["ansible_ssh_common_args"] = options

            if cidr := vm.get("network_address"):
                host_vars["network_cidr"] = cidr
                address = str(ipaddress.ip_interface(cidr).ip)
                host_vars["network_address"] = address

            if host_vars:
                self.inventory["_meta"]["hostvars"][vm["name"]] = host_vars

    def groups(self):
        groups = defaultdict(set)
        for name in self.names:
            for group in [p.as_posix() for p in Path(name).parents][:-1]:
                groups[group].add(name)

        if groups:
            self.inventory["all"]["children"] = list(groups.keys())
            for group, hosts in groups.items():
                self.inventory[group] = {"hosts": list(hosts)}


def main():
    ib = InventoryBuilder()
    ib.hostvars()
    ib.groups()

    print(json.dumps(ib.inventory))


if __name__ == '__main__':
    main()
