use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

use rand::Rng;
use tera::{Context, Tera};

use crate::cache::Cache;
use crate::config::Config;
use crate::images;
use crate::socket;
use crate::specified_by::SpecifiedBy;
use crate::vm_config::VMConfig;
use crate::{Error, Result};

pub fn create(config: &Config, name: &str, image: Option<&str>) -> Result<()> {
    let image = image.unwrap_or(&config.images.default);
    let vm_dir = config.vms_dir.join(name);
    let image_path = images::path(&config.images.directory, &image)?;
    let vm_disk = vm_dir.join(format!("{}.qcow2", name));
    let vml_path = vm_dir.join("vml.toml");

    fs::create_dir_all(&vm_dir)?;
    fs::copy(&image_path, &vm_disk)?;
    fs::OpenOptions::new().create(true).write(true).open(&vml_path)?;

    Ok(())
}

fn get_random_mac() -> String {
    let mut rng = rand::thread_rng();
    let mac_tail = (0..5).map(|_| rng.gen::<u8>().to_string()).collect::<Vec<_>>().join(":");
    format!("fe:{}", &mac_tail)
}

fn get_available_port() -> Option<String> {
    (23000u16..24000u16)
        .find(|&p| TcpListener::bind(("127.0.0.1", p)).is_ok())
        .map(|a| a.to_string())
}

mod ssh_mod {
    #[derive(Clone, Debug)]
    pub struct SSH {
        host: String,
        options: Vec<String>,
        port: String,
        user: Option<String>,
    }

    impl SSH {
        pub fn new(
            user_network: bool,
            address: &Option<String>,
            options: &Option<Vec<String>>,
            port: &Option<String>,
            user: &Option<String>,
        ) -> Option<SSH> {
            let host = if let Some(address) = address {
                address.to_string()
            } else if user_network {
                "localhost".to_string()
            } else {
                return None;
            };

            let port = if let Some(port) = port {
                port.to_string()
            } else {
                return None;
            };

            let options =
                if let Some(options) = options { options.to_owned() } else { Vec::new() };

            let user = user.to_owned();

            Some(SSH { host, options, port, user })
        }

        pub fn user_host(&self, user: &Option<&str>) -> String {
            if let Some(user) = user {
                format!("{}@{}", user, self.host)
            } else if let Some(user) = &self.user {
                format!("{}@{}", user, self.host)
            } else {
                self.host.to_owned()
            }
        }

        pub fn options(&self) -> Vec<&str> {
            let mut options = Vec::with_capacity(self.options.len() * 2);
            for option in &self.options {
                options.push("-o");
                options.push(&option);
            }
            options
        }

        pub fn port(&self) -> &str {
            &self.port
        }
    }
}

use ssh_mod::SSH;

#[derive(Clone, Debug)]
pub struct VM {
    address: Option<String>,
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
    nproc: String,
    specified_by: SpecifiedBy,
    pid: Option<i32>,
    ssh: Option<SSH>,
    tags: HashSet<String>,
    tap: Option<String>,
    user_network: bool,
    vml_directory: PathBuf,
}

impl VM {
    pub fn from_config(config: &Config, name: &str) -> Result<VM> {
        let directory = config.vms_dir.join(name);
        let config_path = directory.join("vml.toml");
        let vm_config = VMConfig::new(&config_path)?;

        VM::from_config_vm_config(config, name, &vm_config)
    }

    pub fn from_config_vm_config(config: &Config, name: &str, vm_config: &VMConfig) -> Result<VM> {
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

        let tap = vm_config.tap;
        let user_network =
            vm_config.user_network.unwrap_or(tap.is_none() && config.default.user_network);

        let address = vm_config.address;
        let cloud_init_image =
            vm_config.cloud_init_image.or_else(|| config.default.cloud_init_image.to_owned());
        let data = vm_config.data.unwrap_or_else(HashMap::new);
        let disk = directory.join(vm_config.disk.unwrap_or_else(|| {
            let mut dp = PathBuf::from(&name);
            dp.set_extension("qcow2");
            dp.file_name().unwrap().into()
        }));
        if !disk.is_file() {
            return Err(Error::disk_does_not_exists(&disk.to_string_lossy(), &name));
        }
        let display = vm_config.display.or_else(|| config.default.display.to_owned());
        let memory = vm_config.memory.unwrap_or_else(|| config.default.memory.to_string());
        let minimum_disk_size =
            vm_config.minimum_disk_size.or_else(|| config.default.minimum_disk_size.to_owned());
        let minimum_disk_size = minimum_disk_size.map(|s| s.get_bytes());
        let nproc = vm_config.nproc.unwrap_or_else(|| config.default.nproc.to_owned()).to_string();
        let ssh_port = vm_config
            .ssh_port
            .or_else(|| {
                if user_network {
                    config.default.ssh_port_user_network.to_owned()
                } else {
                    config.default.ssh_port.to_owned()
                }
            })
            .map(|p| p.to_string());
        let ssh_options = vm_config.ssh_options.or_else(|| config.default.ssh_options.to_owned());
        let ssh_user = vm_config.ssh_user.or_else(|| config.default.ssh_user.to_owned());
        let tags = vm_config.tags.unwrap_or_else(HashSet::new);

        let ssh = SSH::new(user_network, &address, &ssh_options, &ssh_port, &ssh_user);

        Ok(VM {
            address,
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
            pid: None,
            nproc,
            specified_by,
            ssh,
            tags,
            tap,
            user_network,
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

    pub fn has_parent(&self, parent: &str) -> bool {
        self.name_path.starts_with(parent)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    pub fn has_common_tags(&self, tags: &HashSet<String>) -> bool {
        self.tags.intersection(tags).any(|_| true)
    }

    pub fn specify(&mut self, specified_by: SpecifiedBy) {
        if self.specified_by < specified_by {
            self.specified_by = specified_by;
        }
    }

    pub fn start(&self, cloud_init: bool, drives: &[&str]) -> Result<()> {
        #[cfg(debug_assertions)]
        println!("Strart vm {:?}", self.name);
        let mut kvm = Command::new("kvm");

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

        if cloud_init {
            if let Some(image) = &self.cloud_init_image {
                if !image.is_file() {
                    return Err(Error::CloudInitImageDoesNotExists(image.to_owned()));
                }
                kvm.args(&[
                    "-drive",
                    &format!(
                        "file={},if=virtio,format=raw,force-share=on,read-only=on",
                        &image.to_string_lossy()
                    ),
                ]);
            }
        }

        for drive in drives {
            kvm.args(&["-drive", drive]);
        }

        if let Some(ssh) = &self.ssh {
            if self.user_network {
                let port = ssh.port().to_string();
                let port = if port == "random" { get_available_port().unwrap() } else { port };
                self.cache.store("port", &port)?;
                kvm.args(&["-nic", &format!("user,hostfwd=tcp::{}-:22", port)]);
            }
        }

        if let Some(tap) = &self.tap {
            let mac = get_random_mac();
            kvm.args(&["-nic", &format!("tap,ifname={},script=no,mac={}", tap, mac)]);
        }

        if let Some(size) = &self.minimum_disk_size {
            try_resize(&self.disk, *size)?;
        }

        #[cfg(debug_assertions)]
        println!("{:?}", &kvm);
        kvm.spawn().map_err(|e| Error::executable("socat", &e.to_string()))?.wait()?;

        Ok(())
    }

    pub fn stop(&self, force: bool) -> Result<()> {
        #[cfg(debug_assertions)]
        println!("Stop vm {:?}", self.name);

        if let Some(pid) = self.pid {
            if force {
                Command::new("kill")
                    .args(&[pid.to_string()])
                    .spawn()
                    .map_err(|e| Error::executable("kill", &e.to_string()))?;
                #[cfg(debug_assertions)]
                println!("Kill {}", pid);
            } else {
                self.monitor_command("quit")?;
            }
        } else {
            return Err(Error::VMHasNoPid(self.name.to_string()));
        }

        Ok(())
    }

    pub fn ssh(
        &self,
        user: &Option<&str>,
        ssh_options: &[&str],
        cmd: &Option<Vec<&str>>,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        println!("SSH to vm {:?}", self.name);

        let self_ssh =
            self.ssh.as_ref().ok_or_else(|| Error::VMHasNoSSH(self.name.to_string()))?;

        let mut ssh_cmd = Command::new("ssh");

        ssh_cmd.args(self_ssh.options());

        let port = self_ssh.port().to_string();
        let port = if port == "random" { self.cache.load("port")? } else { port };
        ssh_cmd.args(&["-p", &port]);

        ssh_cmd.args(ssh_options);

        ssh_cmd.arg(self_ssh.user_host(&user));

        if let Some(cmd) = cmd {
            let cmd = self.tera_renders(cmd, "ssh commands")?;
            ssh_cmd.args(cmd);
        }

        #[cfg(debug_assertions)]
        println!("{:?}", &ssh_cmd);
        ssh_cmd.spawn().map_err(|e| Error::executable("ssh", &e.to_string()))?.wait()?;

        Ok(())
    }

    pub fn tera_render(&self, template: &str, place: &str) -> Result<String> {
        let mut context = Context::new();
        context.insert("address", &self.address);
        context.insert("data", &self.data);
        context.insert("disk", &self.disk);
        let n = self.names[self.names.len() - 1].to_string();
        context.insert("n", &n);
        context.insert("h", &self.hyphenized());
        context.insert("name", &self.name);
        context.insert("tap", &self.tap);
        context.insert("user_network", &self.user_network);
        Tera::one_off(template, &context, false)
            .map_err(|e| Error::template(place, &e.to_string()))
    }

    pub fn tera_renders(&self, templates: &[&str], place: &str) -> Result<Vec<String>> {
        let mut strings = Vec::with_capacity(templates.len());

        for template in templates {
            strings.push(self.tera_render(template, place)?);
        }

        Ok(strings)
    }

    pub fn tera_render_file(
        &self,
        template_file: &str,
        rendered_file: &str,
        place: &str,
    ) -> Result<()> {
        let template = fs::read_to_string(template_file)?;
        let rendered = self.tera_render(&template, place)?;
        fs::write(rendered_file, &rendered.as_bytes())?;

        Ok(())
    }

    fn rsync_to_from(
        &self,
        to: bool,
        user: &Option<&str>,
        rsync_options: &[&str],
        sources: &[&str],
        destination: &Option<&str>,
    ) -> Result<()> {
        let mut ssh_cmd = vec!["ssh"];

        let sources = self.tera_renders(sources, "rsync sources")?;

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
                let destination = self.tera_render(destination, "rsync destination")?;
                rsync.arg(&format!("{}:{}", user_host, destination));
            }
        } else {
            let sources = sources.join(" ");
            rsync.arg(&format!("{}:{}", user_host, sources));
            if let Some(destination) = destination {
                let destination = self.tera_render(destination, "rsync destination")?;
                rsync.arg(destination);
            }
        }

        #[cfg(debug_assertions)]
        println!("{:#?}", &rsync);
        rsync.spawn().map_err(|e| Error::executable("rsync", &e.to_string()))?.wait()?;

        Ok(())
    }

    pub fn rsync_to(
        &self,
        user: &Option<&str>,
        rsync_options: &[&str],
        sources: &[&str],
        destination: &Option<&str>,
    ) -> Result<()> {
        self.rsync_to_from(true, user, rsync_options, sources, destination)
    }

    pub fn rsync_to_template(
        &self,
        user: &Option<&str>,
        rsync_options: &[&str],
        template: &str,
        destination: &Option<&str>,
    ) -> Result<()> {
        let tmp_dir = tempfile::tempdir().expect("can't create tmp file");
        let tmp_name = tmp_dir.path().join(template).to_string_lossy().to_string();
        let sources = [tmp_name.as_str()];
        self.tera_render_file(template, &tmp_name, "rsync_to_template")?;
        self.rsync_to_from(true, user, rsync_options, &sources, destination)
    }

    pub fn rsync_from(
        &self,
        user: &Option<&str>,
        rsync_options: &[&str],
        sources: &[&str],
        destination: &Option<&str>,
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

    pub fn monitor_command(&self, command: &str) -> Result<Option<String>> {
        let command = &format!("{}\n", command);
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

fn image_size(image: &PathBuf) -> Result<u128> {
    let out = Command::new("qemu-img")
        .args(&["info", "--output=json", &image.to_string_lossy()])
        .output()?;

    let out =
        String::from_utf8(out.stdout).map_err(|e| Error::other("from_utf8", &e.to_string()))?;

    let parsed = json::parse(&out).map_err(|e| Error::other("json", &e.to_string()))?;

    if let Some(size) = parsed["virtual-size"].as_u64() {
        return Ok(size.into());
    }

    Err(Error::other("parse qemu-img out", "can't read virtual-size as u128"))
}

fn try_resize(image: &PathBuf, size: u128) -> Result<()> {
    let current_size = image_size(image)?;

    if current_size < size {
        Command::new("qemu-img")
            .args(&["resize", &image.to_string_lossy(), &size.to_string()])
            .spawn()
            .map_err(|e| Error::executable("qemu-img", &e.to_string()))?
            .wait()?;
    }

    Ok(())
}
