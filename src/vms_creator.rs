use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::PathBuf;

use procfs::process;
use procfs::process::FDTarget;
use walkdir::WalkDir;

use crate::config::Config;
use crate::specified_by::SpecifiedBy;
use crate::vm_config::VMConfig;
use crate::VM;
use crate::{Error, Result};

#[derive(Clone, Debug)]
pub enum WithPid {
    Option,
    Error,
    Filter,
    Without,
}

#[derive(Clone, Debug)]
pub struct VMsCreator<'a> {
    all: bool,
    config: &'a Config,
    error_on_empty: bool,
    names: HashSet<String>,
    parents: HashSet<String>,
    tags: HashSet<String>,
    vm_config: Option<String>,
    with_pid: Option<WithPid>,
}

impl<'a> VMsCreator<'a> {
    pub fn new(config: &'a Config) -> VMsCreator {
        let all = false;
        let error_on_empty = false;
        let names = HashSet::new();
        let parents = HashSet::new();
        let tags = HashSet::new();
        let vm_config = None;
        let with_pid = None;
        VMsCreator { all, config, error_on_empty, names, parents, tags, vm_config, with_pid }
    }

    pub fn vm_config(&mut self, vm_config: &str) {
        self.vm_config = Some(vm_config.to_owned());
    }

    pub fn minimal_vm_config(&mut self) {
        self.vm_config = Some(VMConfig::minimal_config_string());
    }

    pub fn all(&mut self) {
        self.all = true;
        self.names = HashSet::new();
        self.parents = HashSet::new();
        self.tags = HashSet::new();
    }

    pub fn error_on_empty(&mut self) {
        self.error_on_empty = true;
    }

    pub fn name(&mut self, name: &str) {
        self.all = false;
        self.names.insert(name.to_string());
    }

    pub fn names(&mut self, names: &[&str]) {
        self.all = false;
        let names: HashSet<String> = names.iter().cloned().map(|t| t.to_string()).collect();
        self.names.extend(names);
    }

    pub fn parent(&mut self, parent: &str) {
        self.all = false;
        self.parents.insert(parent.to_string());
    }

    pub fn parents(&mut self, parents: &[&str]) {
        self.all = false;
        let parents: HashSet<String> = parents.iter().cloned().map(|t| t.to_string()).collect();
        self.parents.extend(parents);
    }

    pub fn tag(&mut self, tag: &str) {
        self.all = false;
        self.tags.insert(tag.to_string());
    }

    pub fn tags(&mut self, tags: &[&str]) {
        self.all = false;
        let tags: HashSet<String> = tags.iter().cloned().map(|t| t.to_string()).collect();
        self.tags.extend(tags);
    }

    pub fn with_pid(&mut self, with_pid: WithPid) {
        self.with_pid = Some(with_pid);
    }

    pub fn create(&self) -> Result<Vec<VM>> {
        let mut vms: HashMap<PathBuf, VM> = HashMap::new();
        let vms_dir = &self.config.vms_dir;

        for entry in WalkDir::new(vms_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() && entry.path().join("vml.toml").exists() {
                let name = entry
                    .path()
                    .strip_prefix(vms_dir)
                    .expect("prefix is not prefix")
                    .to_string_lossy();
                let mut vm = VM::from_config(self.config, &name)?;

                let mut inser_vm = false;
                if self.names.contains(&vm.name) {
                    vm.specify(SpecifiedBy::Name);
                    inser_vm = true;
                } else if let Some(parent) = self.parents.iter().find(|p| vm.has_parent(p)) {
                    vm.specify(SpecifiedBy::Parent(parent.trim_end_matches('/').to_owned()));
                    inser_vm = true;
                } else if vm.has_common_tags(&self.tags) {
                    vm.specify(SpecifiedBy::Tag);
                    inser_vm = true;
                } else if self.all {
                    vm.specify(SpecifiedBy::All);
                    inser_vm = true;
                }

                if inser_vm {
                    let disk = vm.get_disk().to_owned();
                    if let Some(vm_config) = &self.vm_config {
                        let vm_config = vm.tera_render(&vm_config, "vms_creator:create")?;
                        let vm_config = VMConfig::from_str(&vm_config)?;
                        vm = VM::from_config_vm_config(self.config, &name, &vm_config)?
                    }

                    vms.insert(disk, vm);
                }
            }
        }

        let result_vms: Result<Vec<VM>> = if let Some(with_pid) = &self.with_pid {
            let mut with_pid_vms: HashSet<String> = HashSet::new();
            for proc in process::all_processes()
                .map_err(|e| Error::Other("process:".to_string(), e.to_string()))?
            {
                if let Ok(path) = proc.exe() {
                    if path.file_name() == Some(OsStr::new("qemu-system-x86_64")) {
                        if let Ok(fds) = proc.fd() {
                            for fd in fds {
                                if let FDTarget::Path(f) = fd.target {
                                    if let Some(vm) = vms.get_mut(&f) {
                                        vm.set_pid(proc.pid);
                                        with_pid_vms.insert(vm.name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            match with_pid {
                WithPid::Option => Ok(vms.values().cloned().collect()),
                WithPid::Filter => Ok(vms.values().filter(|v| v.has_pid()).cloned().collect()),
                WithPid::Error => {
                    if let Some(vm) = vms.values().find(|v| !v.has_pid()) {
                        Err(Error::VMHasNoPid(vm.name.to_string()))
                    } else {
                        Ok(vms.values().cloned().collect())
                    }
                }
                WithPid::Without => Ok(vms.values().filter(|v| !v.has_pid()).cloned().collect()),
            }
        } else {
            Ok(vms.values().cloned().collect())
        };

        if let Ok(result_vms) = &result_vms {
            if self.error_on_empty && result_vms.is_empty() {
                return Err(Error::EmptyVMsList);
            }
        }

        result_vms
    }
}
