use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::hash::{Hash, Hasher};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

use rand::Rng;
use tera::Context;

use crate::cache::Cache;
use crate::cloud_init;
use crate::config::Config;
use crate::config::CreateExistsAction;
use crate::images;
use crate::net::{ConfigNet, Net};
use crate::socket;
use crate::specified_by::SpecifiedBy;
use crate::ssh::SSH;
use crate::template;
use crate::vm_config::VMConfig;
use crate::{Error, Result};

pub fn exists<S: AsRef<str>>(config: &Config, name: S) -> bool {
    let vm_dir = config.vms_dir.join(name.as_ref());
    let vml_path = vm_dir.join("vml.toml");
    vml_path.exists()
}

pub fn create<S: AsRef<str>>(
    config: &Config,
    name: S,
    image: Option<&str>,
    exists_action: CreateExistsAction,
) -> Result<()> {
    let name = name.as_ref();
    if exists(config, name) {
        match exists_action {
            CreateExistsAction::Ignore => return Ok(()),
            CreateExistsAction::Fail => return Err(Error::CreateExistingVM(name.to_string())),
            CreateExistsAction::Replace => (),
        }
    }

    let image = image.unwrap_or(&config.images.default);
    let vm_dir = config.vms_dir.join(name);
    let image_path = if image.starts_with('/') {
        PathBuf::from(image)
    } else {
        let mut images_dirs = vec![&config.images.directory];
        images_dirs.extend(config.images.other_directories_ro.iter());
        match images::find(&images_dirs, &image) {
            Ok(image_path) => image_path,
            Err(error) => {
                if matches!(error, Error::ImageDoesNotExists(_)) && config.commands.create.pull {
                    images::pull(&config.images.directory, &image)?
                } else {
                    return Err(error);
                }
            }
        }
    };
    let vm_disk = vm_dir.join("disk.qcow2");
    let vml_path = vm_dir.join("vml.toml");

    fs::create_dir_all(&vm_dir)?;
    fs::copy(&image_path, &vm_disk)?;
    fs::OpenOptions::new().create(true).write(true).open(&vml_path)?;

    Ok(())
}

fn get_random_mac() -> String {
    let mut rng = rand::thread_rng();
    let mac_tail =
        (0..5).map(|_| format!("{:02x}", rng.gen::<u8>())).collect::<Vec<_>>().join(":");
    format!("fe:{}", &mac_tail)
}

fn get_available_port() -> Option<String> {
    (23000u16..24000u16)
        .find(|&p| TcpListener::bind(("127.0.0.1", p)).is_ok())
        .map(|a| a.to_string())
}

#[derive(Clone, Debug)]
pub struct VM {
    cache: Cache,
    cloud_init_image: Option<PathBuf>,
    data: HashMap<String, String>,
    directory: PathBuf,
    disk: PathBuf,
    display: Option<String>,
    memory: String,
    monitor: PathBuf,
    minimum_disk_size: Option<u128>,
    pub name: String,
    name_path: PathBuf,
    names: Vec<String>,
    net: Option<Net>,
    nproc: String,
    specified_by: SpecifiedBy,
    pid: Option<i32>,
    ssh: Option<SSH>,
    tags: HashSet<String>,
    vml_directory: PathBuf,
}

impl VM {
    pub fn from_config<S: AsRef<str>>(config: &Config, name: S) -> Result<VM> {
        let name = name.as_ref();
        let directory = config.vms_dir.join(name);
        let config_path = directory.join("vml.toml");
        let vm_config = VMConfig::new(&config_path)?;

        VM::from_config_vm_config(config, name, &vm_config)
    }

    pub fn from_config_vm_config<S: AsRef<str>>(
        config: &Config,
        name: S,
        vm_config: &VMConfig,
    ) -> Result<VM> {
        let name = name.as_ref();
        let directory = config.vms_dir.join(name);
        let vml_directory = directory.join(".vml");
        let vm_config = vm_config.to_owned();
        let name = vm_config.name.unwrap_or_else(|| name.to_string());
        let name_path = PathBuf::from(&name);
        let names: Vec<String> =
            name_path.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
        let cache = Cache::new(&name, &vml_directory.join("cache"))?;
        let monitor = vml_directory.join("monitor.socket");

        let specified_by = SpecifiedBy::All;

        let cloud_init_image =
            vm_config.cloud_init_image.or_else(|| config.default.cloud_init_image.to_owned());
        let data = vm_config.data.unwrap_or_else(HashMap::new);
        let disk = directory.join(vm_config.disk.unwrap_or_else(|| PathBuf::from("disk.qcow2")));
        if !disk.is_file() {
            return Err(Error::disk_does_not_exists(&disk.to_string_lossy(), &name));
        }
        let display = vm_config.display.or_else(|| config.default.display.to_owned());
        let memory = vm_config.memory.unwrap_or_else(|| config.default.memory.to_string());
        let minimum_disk_size =
            vm_config.minimum_disk_size.or_else(|| config.default.minimum_disk_size.to_owned());
        let minimum_disk_size = minimum_disk_size.map(|s| s.get_bytes());
        let nproc = vm_config.nproc.unwrap_or_else(|| config.default.nproc.to_owned()).to_string();
        let tags = vm_config.tags.unwrap_or_else(HashSet::new);

        let mut net_config = vm_config.net.updated(&config.default.net);
        if let ConfigNet::Tap { nameservers: ref mut nameservers @ None, .. } = net_config {
            *nameservers = config.nameservers.to_owned();
        }
        let net = Net::new(&net_config)?;

        let ssh_config = vm_config.ssh.updated(&config.default.ssh);
        let ssh = SSH::new(&ssh_config, &net_config);

        Ok(VM {
            cache,
            cloud_init_image,
            data,
            directory,
            disk,
            display,
            memory,
            monitor,
            minimum_disk_size,
            name,
            name_path,
            names,
            net,
            pid: None,
            nproc,
            specified_by,
            ssh,
            tags,
            vml_directory,
        })
    }

    pub fn hyphenized(&self) -> String {
        self.names.join("-")
    }

    pub fn get_disk(&self) -> &PathBuf {
        &self.disk
    }

    pub fn set_pid(&mut self, pid: i32) {
        self.pid = Some(pid);
    }

    pub fn has_pid(&self) -> bool {
        self.pid.is_some()
    }

    pub fn has_parent<S: AsRef<str>>(&self, parent: S) -> bool {
        self.name_path.starts_with(parent.as_ref())
    }

    pub fn has_tag<S: AsRef<str>>(&self, tag: S) -> bool {
        self.tags.contains(tag.as_ref())
    }

    pub fn has_common_tags(&self, tags: &HashSet<String>) -> bool {
        self.tags.intersection(tags).any(|_| true)
    }

    pub fn specify(&mut self, specified_by: SpecifiedBy) {
        if self.specified_by < specified_by {
            self.specified_by = specified_by;
        }
    }

    pub fn start<S: AsRef<OsStr>>(&self, cloud_init: bool, drives: &[S]) -> Result<()> {
        #[cfg(debug_assertions)]
        eprintln!("Strart vm {:?}", self.name);
        let mut kvm = Command::new("kvm");
        let mut context = self.context();

        kvm.args(&["-m", &self.memory])
            .args(&["-cpu", "host"])
            .args(&["-smp", &self.nproc])
            .args(&["-drive", &format!("file={},if=virtio", self.disk.to_string_lossy())])
            .args(&["-monitor", &format!("unix:{},server,nowait", self.monitor.to_string_lossy())])
            .arg("-daemonize")
            .current_dir(&self.directory);

        if let Some(display) = &self.display {
            kvm.args(&["-display", display]);
        }

        for drive in drives {
            kvm.arg("-drive").arg(drive);
        }

        if let Some(net) = &self.net {
            match net {
                Net::User => {
                    let hostfwd = if let Some(ssh) = &self.ssh {
                        let port = ssh.port().to_string();
                        let port =
                            if port == "random" { get_available_port().unwrap() } else { port };
                        self.cache.store("port", &port)?;
                        format!(",hostfwd=tcp::{}-:22", port)
                    } else {
                        "".to_string()
                    };
                    kvm.args(&["-nic", &format!("user{}", hostfwd)]);
                }
                Net::Tap { address, nameservers, tap, .. } => {
                    context.insert("address", &address);
                    context.insert("gateway4", &net.gateway4());
                    context.insert("gateway6", &net.gateway6());
                    context.insert("tap", &tap);
                    context.insert("nameservers", &nameservers);
                    let mac = get_random_mac();
                    context.insert("mac", &mac);
                    kvm.args(&["-nic", &format!("tap,ifname={},script=no,mac={}", tap, mac)]);
                }
            }
        }

        if cloud_init {
            let image = if let Some(image) = &self.cloud_init_image {
                if !image.is_file() {
                    return Err(Error::CloudInitImageDoesNotExists(image.to_owned()));
                }

                image.to_owned()
            } else {
                if let Some(ssh) = &self.ssh {
                    if ssh.has_key() {
                        let keys = ssh.ensure_keys(&self.vml_directory.join("ssh"))?;
                        context.insert("ssh_authorized_keys", &keys.authorized_keys());

                        let users = if let Some(user) = ssh.user() { vec![user] } else { vec![] };
                        context.insert("users", &users);
                    }
                }

                cloud_init::generate_data(&context, &self.vml_directory.join("cloud-init"))?
            };

            kvm.args(&[
                "-drive",
                &format!(
                    "file={},if=virtio,format=raw,force-share=on,read-only=on",
                    &image.to_string_lossy()
                ),
            ]);
        }

        if let Some(size) = &self.minimum_disk_size {
            try_resize(&self.disk, *size)?;
        }

        #[cfg(debug_assertions)]
        eprintln!("{:?}", &kvm);
        kvm.spawn().map_err(|e| Error::executable("kvm", &e.to_string()))?.wait()?;

        Ok(())
    }

    pub fn stop(&mut self, force: bool) -> Result<()> {
        #[cfg(debug_assertions)]
        eprintln!("Stop vm {:?}", self.name);

        if let Some(pid) = self.pid {
            if force {
                Command::new("kill")
                    .args(&[pid.to_string()])
                    .spawn()
                    .map_err(|e| Error::executable("kill", &e.to_string()))?;
                #[cfg(debug_assertions)]
                eprintln!("Kill {}", pid);
            } else {
                self.monitor_command("quit")?;
            }

            self.pid = None;
        } else {
            return Err(Error::VMHasNoPid(self.name.to_string()));
        }

        Ok(())
    }

    pub fn ssh<U: AsRef<str>, O: AsRef<OsStr>, F: AsRef<OsStr>, C: AsRef<str>>(
        &self,
        user: &Option<U>,
        ssh_options: &[O],
        ssh_flags: &[F],
        cmd: &Option<Vec<C>>,
    ) -> Result<Option<i32>> {
        #[cfg(debug_assertions)]
        eprintln!("SSH to vm {:?}", self.name);

        let self_ssh =
            self.ssh.as_ref().ok_or_else(|| Error::VMHasNoSSH(self.name.to_string()))?;

        let mut ssh_cmd = Command::new("ssh");

        ssh_cmd.args(self_ssh.options());

        let port = self_ssh.port().to_string();
        let port = if port == "random" { self.cache.load("port")? } else { port };
        ssh_cmd.args(&["-p", &port]);

        for option in ssh_options {
            ssh_cmd.arg("-o").arg(option.as_ref());
        }

        ssh_cmd.args(ssh_flags);

        ssh_cmd.arg(self_ssh.user_host(user));
        if self_ssh.has_key() {
            let keys = self_ssh.ensure_keys(&self.vml_directory.join("ssh"))?;
            if let Some(pvt_key) = keys.private() {
                ssh_cmd.args(&["-o", "IdentitiesOnly=yes"]);
                ssh_cmd.arg("-i").arg(pvt_key);
            }
        }

        if let Some(cmd) = cmd {
            let cmd = template::renders(&self.context(), cmd, "ssh commands")?;
            ssh_cmd.args(cmd);
        }

        #[cfg(debug_assertions)]
        eprintln!("{:?}", &ssh_cmd);
        let rc =
            ssh_cmd.spawn().map_err(|e| Error::executable("ssh", &e.to_string()))?.wait()?.code();

        Ok(rc)
    }

    pub fn context(&self) -> Context {
        let mut context = Context::new();
        context.insert("data", &self.data);
        context.insert("disk", &self.disk);
        let n = self.names[self.names.len() - 1].to_string();
        context.insert("n", &n);
        context.insert("h", &self.hyphenized());
        context.insert("name", &self.name);

        context
    }

    fn rsync_to_from<U: AsRef<str>, O: AsRef<OsStr>, S: AsRef<str>, D: AsRef<str>>(
        &self,
        to: bool,
        user: &Option<U>,
        rsync_options: &[O],
        sources: &[S],
        destination: &Option<D>,
    ) -> Result<()> {
        let mut ssh_cmd = vec!["ssh"];

        let context = self.context();
        let sources = template::renders(&context, sources, "rsync sources")?;

        let self_ssh =
            self.ssh.as_ref().ok_or_else(|| Error::VMHasNoSSH(self.name.to_string()))?;

        ssh_cmd.extend(&self_ssh.options());

        let port = self_ssh.port().to_string();
        let port = if port == "random" { self.cache.load("port")? } else { port };
        ssh_cmd.extend(&["-p", &port]);

        let user_host = self_ssh.user_host(&user);

        let mut rsync = Command::new("rsync");
        rsync.arg("-e").arg(ssh_cmd.join(" "));
        rsync.args(rsync_options);

        if to {
            rsync.args(sources);
            if let Some(destination) = destination {
                let destination = template::render(&context, destination, "rsync destination")?;
                rsync.arg(&format!("{}:{}", user_host, destination));
            }
        } else {
            let sources = sources.join(" ");
            rsync.arg(&format!("{}:{}", user_host, sources));
            if let Some(destination) = destination {
                let destination = template::render(&context, destination, "rsync destination")?;
                rsync.arg(destination);
            }
        }

        #[cfg(debug_assertions)]
        eprintln!("{:#?}", &rsync);
        rsync.spawn().map_err(|e| Error::executable("rsync", &e.to_string()))?.wait()?;

        Ok(())
    }

    pub fn rsync_to<U: AsRef<str>, O: AsRef<OsStr>, S: AsRef<str>, D: AsRef<str>>(
        &self,
        user: &Option<U>,
        rsync_options: &[O],
        sources: &[S],
        destination: &Option<D>,
    ) -> Result<()> {
        self.rsync_to_from(true, user, rsync_options, sources, destination)
    }

    pub fn rsync_to_template<U: AsRef<str>, O: AsRef<OsStr>, T: AsRef<str>, D: AsRef<str>>(
        &self,
        user: &Option<U>,
        rsync_options: &[O],
        template_str: T,
        destination: &Option<D>,
    ) -> Result<()> {
        let tmp_dir = tempfile::tempdir().expect("can't create tmp file");
        let tmp_name = tmp_dir.path().join(template_str.as_ref()).to_string_lossy().to_string();
        let sources = [tmp_name.as_str()];
        template::render_file(
            &self.context(),
            template_str.as_ref(),
            &tmp_name,
            "rsync_to_template",
        )?;
        self.rsync_to_from(true, user, rsync_options, &sources, destination)
    }

    pub fn rsync_from<U: AsRef<str>, O: AsRef<OsStr>, S: AsRef<str>, D: AsRef<str>>(
        &self,
        user: &Option<U>,
        rsync_options: &[O],
        sources: &[S],
        destination: &Option<D>,
    ) -> Result<()> {
        self.rsync_to_from(false, user, rsync_options, sources, destination)
    }

    pub fn monitor(&self) -> Result<()> {
        Command::new("socat")
            .arg("-,echo=0,icanon=0")
            .arg(&format!("unix-connect:{}", &self.monitor.to_string_lossy()))
            .spawn()
            .map_err(|e| Error::executable("socat", &e.to_string()))?
            .wait()?;

        Ok(())
    }

    pub fn monitor_command<S: AsRef<str>>(&self, command: S) -> Result<Option<String>> {
        let command = &format!("{}\n", command.as_ref());
        let reply = socket::reply(command.as_bytes(), &self.monitor)?;
        let reply =
            String::from_utf8(reply).map_err(|e| Error::other("from_utf8", &e.to_string()))?;
        let lines: Vec<&str> = reply.lines().collect();
        if lines.len() > 3 {
            lines[2..lines.len() - 1].join("\n");
            if !reply.is_empty() {
                return Ok(Some(reply));
            }
        }

        Ok(None)
    }

    pub fn remove(self) -> Result<()> {
        #[cfg(debug_assertions)]
        eprintln!("Remove vm {:?}", self.name);
        if self.has_pid() {
            return Err(Error::RemoveRuuningVM(self.name));
        }

        fs::remove_dir_all(self.directory)?;

        Ok(())
    }

    pub fn folded_name(&self) -> String {
        match &self.specified_by {
            SpecifiedBy::All | SpecifiedBy::Tag => {
                let ancestors: Vec<&Path> = self.name_path.ancestors().collect();
                let len = ancestors.len();
                if len > 2 {
                    let ancestor = ancestors[len - 2];
                    format!("{}/", ancestor.to_string_lossy())
                } else {
                    self.name.to_owned()
                }
            }
            SpecifiedBy::Parent(parent) => {
                let name_path = self.name_path.strip_prefix(parent).expect("Parent in not prefix");
                let ancestors: Vec<&Path> = name_path.ancestors().collect();
                let len = ancestors.len();
                if len > 2 {
                    let ancestor = ancestors[len - 2];
                    format!("{}/{}/", parent, ancestor.to_string_lossy())
                } else {
                    self.name.to_owned()
                }
            }
            SpecifiedBy::Name => self.name.to_owned(),
        }
    }

    pub fn store_disk<P: AsRef<Path>>(&self, to: P, force: bool) -> Result<()> {
        let to = to.as_ref();

        if self.has_pid() {
            return Err(Error::StoreRunningVM(self.name.to_string()));
        }

        if to.exists() && !force {
            return Err(Error::RewriteExistsPath(to.to_string_lossy().to_string()));
        }

        fs::copy(&self.disk, to)?;

        Ok(())
    }
}

impl PartialEq for VM {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for VM {}

impl Hash for VM {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.name.hash(hasher)
    }
}

fn image_size<S: AsRef<OsStr>>(image: S) -> Result<u128> {
    let out =
        Command::new("qemu-img").args(&["info", "--output=json"]).arg(image.as_ref()).output()?;

    let out =
        String::from_utf8(out.stdout).map_err(|e| Error::other("from_utf8", &e.to_string()))?;

    let parsed = json::parse(&out).map_err(|e| Error::other("json", &e.to_string()))?;

    if let Some(size) = parsed["virtual-size"].as_u64() {
        return Ok(size.into());
    }

    Err(Error::other("parse qemu-img out", "can't read virtual-size as u128"))
}

fn try_resize<S: AsRef<OsStr>>(image: S, size: u128) -> Result<()> {
    let image = image.as_ref();
    let current_size = image_size(image)?;

    if current_size < size {
        Command::new("qemu-img")
            .arg("resize")
            .arg(image)
            .arg(&size.to_string())
            .spawn()
            .map_err(|e| Error::executable("qemu-img", &e.to_string()))?
            .wait()?;
    }

    Ok(())
}
