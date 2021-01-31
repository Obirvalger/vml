use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;

use rand::Rng;
use tera::{Context, Tera};

use crate::cache::Cache;
use crate::config::Config;
use crate::vm_config::VMConfig;
use crate::{Error, Result};

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
    directory: PathBuf,
    disk: PathBuf,
    display: Option<String>,
    memory: String,
    minimum_disk_size: Option<u128>,
    pub name: String,
    name_path: PathBuf,
    nproc: String,
    ssh: Option<SSH>,
    pid: Option<i32>,
    tags: HashSet<String>,
    tap: Option<String>,
    user_network: bool,
}

impl VM {
    pub fn from_config(config: &Config, name: &str) -> Result<VM> {
        let cache = Cache::new(name);
        let directory = config.vms_dir.join(name);
        let config_path = directory.join("vml.toml");
        let vm_config = VMConfig::new(&config_path)?;
        let name = vm_config.name.unwrap_or_else(|| name.to_string());
        let name_path = PathBuf::from(&name);

        let tap = vm_config.tap;
        let user_network =
            vm_config.user_network.unwrap_or(tap.is_none() && config.default.user_network);

        let address = vm_config.address;
        let cloud_init_image =
            vm_config.cloud_init_image.or_else(|| config.default.cloud_init_image.to_owned());
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
            directory,
            disk,
            display,
            memory,
            minimum_disk_size,
            name,
            name_path,
            pid: None,
            nproc,
            ssh,
            tags,
            tap,
            user_network,
        })
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

    pub fn start(&self, cloud_init: bool) -> Result<()> {
        #[cfg(debug_assertions)]
        println!("Strart vm {:?}", self.name);
        let mut kvm = Command::new("kvm");

        kvm.args(&["-m", &self.memory])
            .args(&["-cpu", "host"])
            .args(&["-smp", &self.nproc])
            .args(&["-drive", &format!("file={},if=virtio", self.disk.to_string_lossy())])
            .arg("-daemonize")
            .current_dir(&self.directory);

        if let Some(display) = &self.display {
            kvm.args(&["-display", display]);
        }

        if cloud_init {
            if let Some(image) = &self.cloud_init_image {
                kvm.args(&[
                    "-drive",
                    &format!(
                        "file={},if=virtio,format=raw,force-share=on,read-only=on",
                        &image.to_string_lossy()
                    ),
                ]);
            }
        }

        if let Some(ssh) = &self.ssh {
            if self.user_network {
                let port = ssh.port().to_string();
                let port = if port == "random" { get_available_port().unwrap() } else { port };
                self.cache.store("port", &port);
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
        kvm.spawn()?.wait()?;

        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        #[cfg(debug_assertions)]
        println!("Stop vm {:?}", self.name);

        if let Some(pid) = self.pid {
            Command::new("kill").args(&[pid.to_string()]).spawn()?;
            #[cfg(debug_assertions)]
            println!("Kill {}", pid);
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
        let port = if port == "random" { self.cache.load("port") } else { port };
        ssh_cmd.args(&["-p", &port]);

        ssh_cmd.args(ssh_options);

        ssh_cmd.arg(self_ssh.user_host(&user));

        if let Some(cmd) = cmd {
            let cmd = self.tera_renders(cmd, "ssh commands")?;
            ssh_cmd.args(cmd);
        }

        #[cfg(debug_assertions)]
        println!("{:?}", &ssh_cmd);
        ssh_cmd.spawn()?.wait()?;

        Ok(())
    }

    fn tera_render(&self, template: &str, place: &str) -> Result<String> {
        let mut context = Context::new();
        context.insert("name", &self.name);
        context.insert("address", &self.address);
        context.insert("user_network", &self.user_network);
        Tera::one_off(template, &context, false)
            .map_err(|e| Error::template(place, &e.to_string()))
    }

    fn tera_renders(&self, templates: &[&str], place: &str) -> Result<Vec<String>> {
        let mut strings = Vec::with_capacity(templates.len());

        for template in templates {
            strings.push(self.tera_render(template, place)?);
        }

        Ok(strings)
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
        let port = if port == "random" { self.cache.load("port") } else { port };
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
        rsync.spawn()?.wait()?;

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

    pub fn rsync_from(
        &self,
        user: &Option<&str>,
        rsync_options: &[&str],
        sources: &[&str],
        destination: &Option<&str>,
    ) -> Result<()> {
        self.rsync_to_from(false, user, rsync_options, sources, destination)
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
            .spawn()?
            .wait()?;
    }

    Ok(())
}
