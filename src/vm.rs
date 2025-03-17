use std::collections::{BTreeMap, HashMap, HashSet};
use std::env::consts::ARCH;
use std::ffi::OsStr;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::Context as AnyhowContext;
use anyhow::{bail, Result};
use cmd_lib::run_fun;
use file_lock::{FileLock, FileOptions};
use log::{debug, info};
use procfs::process::Process;
use rand::Rng;
use tera::Context;

use crate::cache::Cache;
use crate::cloud_init;
use crate::config::{config_dir, Config, CreateExistsAction};
use crate::gui::ConfigGui;
use crate::images;
use crate::images::Images;
use crate::net::{ConfigNet, Net};
use crate::socket;
use crate::specified_by::SpecifiedBy;
use crate::ssh::Ssh;
use crate::template;
use crate::vm_config::VMConfig;
use crate::Error;

pub fn exists<S: AsRef<str>>(config: &Config, name: S) -> bool {
    let vm_dir = config.vms_dir.join(name.as_ref());
    let vml_path = vm_dir.join("vml.toml");
    vml_path.exists()
}

pub async fn create<S: AsRef<str>>(
    config: &Config,
    vm_config: &VMConfig,
    name: S,
    image: Option<&str>,
    exists_action: CreateExistsAction,
    available_images: &Images<'_>,
    show_pb: bool,
) -> Result<()> {
    let name = name.as_ref();
    if exists(config, name) {
        match exists_action {
            CreateExistsAction::Ignore => return Ok(()),
            CreateExistsAction::Fail => bail!(Error::CreateExistingVM(name.to_string())),
            CreateExistsAction::Replace => (),
        }
    }

    let image_name = image.unwrap_or(&config.images.default);
    let vm_dir = config.vms_dir.join(name);
    let image_path = if image_name.starts_with('/') {
        PathBuf::from(image_name)
    } else {
        let mut images_dirs = vec![&config.images.directory];
        images_dirs.extend(config.images.other_directories_ro.iter());
        match images::find(&images_dirs, image_name) {
            Ok(image_path) => {
                if config.images.update_on_create {
                    if let Some(image) = available_images.get(image_name) {
                        if image.outdate() {
                            info!("Update {} image", &image.name);
                            image.pull(show_pb).await?;
                        }
                    }
                }
                image_path
            }
            Err(error) => {
                let image_does_not_exist =
                    matches!(error.downcast_ref::<Error>(), Some(Error::ImageDoesNotExists(_)));
                if image_does_not_exist && config.commands.create.pull {
                    let image = available_images.get_result(image_name)?;
                    image.pull(show_pb).await?
                } else {
                    bail!(error);
                }
            }
        }
    };
    let vm_disk = vm_dir.join("disk.qcow2");
    let vml_path = vm_dir.join("vml.toml");

    fs::create_dir_all(&vm_dir)?;
    fs::copy(image_path, vm_disk)?;
    if !vml_path.is_file() {
        let mut vm_config = vm_config.to_owned();
        if let Some(image) = available_images.get(image_name) {
            if vm_config.properties.is_none() && !image.properties.is_empty() {
                vm_config.properties = Some(image.properties.to_owned())
            }

            vm_config.image_name = Some(image.name.to_owned());
        }
        let vm_config_string = toml::to_string(&vm_config).expect("Could not serialize vm config");
        fs::write(&vml_path, vm_config_string)?
    }

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

fn bios_options<S: AsRef<str>>(bios: S, efi: bool) -> Vec<String> {
    if ARCH != "aarch64" && !efi {
        vec![]
    } else {
        vec!["-bios".to_string(), bios.as_ref().to_string()]
    }
}

#[derive(Clone, Debug)]
pub struct VM {
    cache: Cache,
    cloud_init: bool,
    cloud_init_image: Option<PathBuf>,
    cpu_model: String,
    data: HashMap<String, String>,
    directory: PathBuf,
    disk: PathBuf,
    display: Option<String>,
    gui: Option<ConfigGui>,
    memory: String,
    monitor: PathBuf,
    minimum_disk_size: Option<u64>,
    image_name: Option<String>,
    pub name: String,
    name_path: PathBuf,
    names: Vec<String>,
    net: Option<Net>,
    nic_model: String,
    nproc: String,
    specified_by: SpecifiedBy,
    pid: Option<i32>,
    openssh_config: PathBuf,
    qemu_binary: String,
    qemu_arch_options: Vec<String>,
    qemu_bios_options: Vec<String>,
    ssh: Option<Ssh>,
    tags: HashSet<String>,
    vml_directory: PathBuf,
}

impl VM {
    pub fn from_config<S: AsRef<str>>(config: &Config, name: S) -> Result<VM> {
        let name = name.as_ref();
        let directory = config.vms_dir.join(name);
        let config_path = directory.join("vml.toml");
        let mut vm_config = VMConfig::new(&config_path)?;
        if config.config_hierarchy {
            for config_dir in directory.ancestors().skip(1) {
                let config_path = config_dir.join("vml-common.toml");
                if config_path.is_file() {
                    vm_config.update(&VMConfig::new(&config_path)?);
                }

                if config_dir == config.vms_dir {
                    break;
                }
            }
        }

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
        let cache = Cache::new(&vml_directory.join("cache"))?;
        let monitor = vml_directory.join("monitor.socket");

        let specified_by = SpecifiedBy::All;

        let cloud_init = vm_config.cloud_init.unwrap_or(config.default.cloud_init);
        let cloud_init_image =
            vm_config.cloud_init_image.or_else(|| config.default.cloud_init_image.to_owned());
        let cpu_model =
            vm_config.cpu_model.unwrap_or_else(|| config.default.cpu_model.to_string());
        let data = vm_config.data.unwrap_or_default();
        let disk = directory.join(vm_config.disk.unwrap_or_else(|| PathBuf::from("disk.qcow2")));
        if !disk.is_file() {
            bail!("disk `{}` for vm `{}` not found", &disk.display(), &name);
        }
        let mut default_gui = config.default.gui.to_owned();
        let mut default_display = config.default.display.to_owned();
        let properties = vm_config.properties.unwrap_or_default();
        if properties.contains("gui") {
            default_display = Some("gtk".to_string())
        } else {
            default_gui = None;
        }
        let display = vm_config.display.or(default_display);
        let gui = vm_config.gui.or(default_gui);
        let memory = vm_config.memory.unwrap_or_else(|| config.default.memory.to_string());
        let minimum_disk_size =
            vm_config.minimum_disk_size.or_else(|| config.default.minimum_disk_size.to_owned());
        let minimum_disk_size = minimum_disk_size.map(|s| s.get_bytes());
        let nic_model =
            vm_config.nic_model.unwrap_or_else(|| config.default.nic_model.to_string());
        let nproc = vm_config.nproc.unwrap_or_else(|| config.default.nproc.to_owned()).to_string();
        let tags = vm_config.tags.unwrap_or_default();

        let mut net_config = match vm_config.net {
            None => config.default.net.to_owned(),
            Some(net) => net.updated(&config.default.net),
        };
        if let ConfigNet::Tap { nameservers: ref mut nameservers @ None, .. } = net_config {
            config.nameservers.clone_into(nameservers);
        }
        let net = Net::new(&net_config)?;

        let qemu_binary =
            vm_config.qemu_binary.unwrap_or_else(|| config.default.qemu_binary.to_string());

        let qemu_arch_options = vm_config
            .qemu_arch_options
            .unwrap_or_else(|| config.default.qemu_arch_options.to_owned());

        let qemu_bios =
            vm_config.qemu_bios.unwrap_or_else(|| config.default.qemu_bios.to_string());

        let qemu_bios_options = bios_options(qemu_bios, properties.contains("efi"));

        let ssh_config = match vm_config.ssh {
            None => config.default.ssh.to_owned(),
            Some(ssh) => ssh.updated(&config.default.ssh),
        };
        let ssh = Ssh::new(&ssh_config, &net_config);

        let openssh_config = config.openssh_config.vm_configs_dir.join(&name);

        Ok(VM {
            cache,
            cloud_init,
            cloud_init_image,
            cpu_model,
            data,
            directory,
            disk,
            display,
            gui,
            memory,
            monitor,
            minimum_disk_size,
            image_name: vm_config.image_name.to_owned(),
            name,
            name_path,
            names,
            net,
            pid: None,
            nic_model,
            nproc,
            openssh_config,
            qemu_binary,
            qemu_arch_options,
            qemu_bios_options,
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

    pub fn start<S: AsRef<OsStr>>(
        &self,
        cloud_init: Option<bool>,
        snapshot: bool,
        drives: &[S],
    ) -> Result<()> {
        debug!("Start vm {:?}", self.name);
        let mut qemu = Command::new(&self.qemu_binary);
        let mut context = self.context();
        let mut user_net = "".to_string();

        if snapshot {
            qemu.arg("-snapshot");
        }

        qemu.args(["-m", &self.memory])
            .arg("--enable-kvm")
            .args(&self.qemu_arch_options)
            .args(&self.qemu_bios_options)
            .args(["-cpu", &self.cpu_model])
            .args(["-smp", &self.nproc])
            .args(["-drive", &format!("file={},if=virtio", self.disk.to_string_lossy())])
            .args(["-monitor", &format!("unix:{},server,nowait", self.monitor.to_string_lossy())])
            .current_dir(&self.directory);

        if let Some(display) = &self.display {
            if display == "console" {
                qemu.args(["-nographic", "-serial", "mon:stdio"]);
            } else {
                qemu.args(["-display", display]);
                qemu.arg("-daemonize");
            }
        } else {
            qemu.arg("-daemonize");
        }

        for drive in drives {
            qemu.arg("-drive").arg(drive);
        }

        if let Some(net) = &self.net {
            match net {
                Net::User => {
                    let hostfwd = if let Some(ssh) = &self.ssh {
                        let host = ssh.host().to_string();
                        let port = ssh.port().to_string();
                        format!(",hostfwd=tcp:{}:{}-:22,model={}", host, port, &self.nic_model)
                    } else {
                        "".to_string()
                    };
                    user_net = format!("user{}", hostfwd);
                }
                Net::Tap { address, nameservers, tap, .. } => {
                    context.insert("address", &address);
                    context.insert("gateway4", &net.gateway4());
                    context.insert("gateway6", &net.gateway6());
                    context.insert("tap", &tap);
                    context.insert("nameservers", &nameservers);
                    let mac = get_random_mac();
                    context.insert("mac", &mac);
                    qemu.args([
                        "-nic",
                        &format!(
                            "tap,ifname={},script=no,mac={},model={}",
                            tap, mac, &self.nic_model
                        ),
                    ]);
                }
            }
        }

        let cloud_init =
            if let Some(cloud_init) = cloud_init { cloud_init } else { self.cloud_init };

        if cloud_init {
            let image = if let Some(image) = &self.cloud_init_image {
                if !image.is_file() {
                    bail!(Error::CloudInitImageDoesNotExists(image.to_owned()));
                }

                image.to_owned()
            } else {
                let mut users = vec![];

                if let Some(gui) = &self.gui {
                    users.push(gui.user.to_string());
                    context.insert("gui", gui);
                }

                if let Some(ssh) = &self.ssh {
                    if ssh.has_key() {
                        let keys = ssh.ensure_keys(&self.vml_directory.join("ssh"))?;
                        context.insert("ssh_authorized_keys", &keys.authorized_keys());

                        if let Some(user) = ssh.user() {
                            users.push(user)
                        }
                    }
                }

                context.insert("users", &users);

                cloud_init::generate_data(&context, &self.vml_directory.join("cloud-init"))?
            };

            qemu.args([
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

        if !user_net.is_empty() {
            if user_net.contains("random") {
                let options = FileOptions::new().create(true).truncate(true).write(true);
                let block = true;
                let lock_path = config_dir().join("port-lock");
                if let Ok(mut filelock) = FileLock::lock(lock_path, block, options) {
                    let port = get_available_port().unwrap();
                    self.cache.store("port", &port)?;
                    let user_net = user_net.replace("random", &port);
                    qemu.args(["-nic", &user_net]);

                    debug!("{:?}", &qemu);
                    let exit_status = qemu
                        .spawn()
                        .with_context(|| {
                            format!("failed to run executable executable {}", &self.qemu_binary)
                        })?
                        .wait()?;

                    if !exit_status.success() {
                        bail!(Error::StartVmFailed(self.name.to_string()));
                    }

                    filelock.file.write_all(port.as_bytes())?;

                    return Ok(());
                }
            } else {
                qemu.args(["-nic", &user_net]);
            };
        }

        debug!("{:?}", &qemu);
        let exit_status = qemu
            .spawn()
            .with_context(|| format!("failed to run executable executable {}", &self.qemu_binary))?
            .wait()?;

        if !exit_status.success() {
            bail!(Error::StartVmFailed(self.name.to_string()));
        }

        Ok(())
    }

    pub fn stop(&mut self, force: bool) -> Result<()> {
        debug!("Stop vm {:?}", self.name);

        fn kill(pid: i32) -> Result<()> {
            Command::new("kill")
                .args(&[pid.to_string()])
                .spawn()
                .context("failed to run executable kill")?;
            debug!("Kill {}", pid);

            Ok(())
        }

        if let Some(pid) = self.pid {
            if force {
                kill(pid)?;
            } else {
                self.monitor_command("system_powerdown")?;
            }

            let mut repeat = 900; // stop after 90 seconds
            let sleep = 100; // milliseconds
            let mut killed = false;
            loop {
                if !killed && repeat <= 0 {
                    kill(pid)?;
                    killed = true;
                }
                if Process::new(pid).is_err() {
                    break;
                } else {
                    thread::sleep(Duration::from_millis(sleep));
                }

                repeat -= 1;
            }

            self.pid = None;
        } else {
            bail!(Error::VMHasNoPid(self.name.to_string()));
        }

        Ok(())
    }

    fn get_ssh_private_key(&self) -> Result<Option<String>> {
        let mut key = None;

        if let Some(self_ssh) = &self.ssh {
            if self_ssh.has_key() {
                let keys = self_ssh.ensure_keys(&self.vml_directory.join("ssh"))?;
                if let Some(pvt_key) = keys.private() {
                    key = Some(pvt_key);
                }
            }
        }

        Ok(key)
    }

    fn get_ssh_port(&self) -> Result<Option<String>> {
        if let Some(self_ssh) = &self.ssh {
            let port = self_ssh.port().to_string();
            let port = if port == "random" { self.cache.load("port")? } else { port };
            Ok(Some(port))
        } else {
            Ok(None)
        }
    }

    pub fn ssh<U: AsRef<str>, O: AsRef<OsStr>, F: AsRef<OsStr>, C: AsRef<str>>(
        &self,
        user: &Option<U>,
        ssh_options: &[O],
        ssh_flags: &[F],
        cmd: &Option<Vec<C>>,
    ) -> Result<Option<i32>> {
        debug!("Ssh to vm {:?}", self.name);

        let self_ssh =
            self.ssh.as_ref().ok_or_else(|| Error::VMHasNoSsh(self.name.to_string()))?;

        let mut ssh_cmd = Command::new("ssh");

        ssh_cmd.args(self_ssh.options());

        if let Some(port) = self.get_ssh_port()? {
            ssh_cmd.args(["-p", &port]);
        }

        for option in ssh_options {
            ssh_cmd.arg("-o").arg(option.as_ref());
        }

        ssh_cmd.args(ssh_flags);

        ssh_cmd.arg(self_ssh.user_host(user));

        if let Some(pvt_key) = self.get_ssh_private_key()? {
            ssh_cmd.args(["-o", "IdentitiesOnly=yes"]);
            ssh_cmd.arg("-i").arg(pvt_key);
        }

        if let Some(cmd) = cmd {
            let cmd = template::renders(&self.context(), cmd, "ssh commands")?;
            ssh_cmd.args(cmd);
        }

        debug!("{:?}", &ssh_cmd);
        let rc = ssh_cmd.spawn().context("failed to run executable ssh")?.wait()?.code();

        Ok(rc)
    }

    pub fn context(&self) -> Context {
        let mut context = Context::new();
        context.insert("data", &self.data);
        context.insert("disk", &self.disk);
        let len = self.names.len();
        let n = self.names[len - 1].to_string();
        context.insert("n", &n);
        context.insert("h", &self.hyphenized());
        context.insert("name", &self.name);
        let hostname = if n.parse::<u128>().is_ok() {
            let host = if len > 1 { &self.names[len - 2] } else { "host" };
            format!("{}-{}", host, &n)
        } else {
            n
        };
        context.insert("hostname", &hostname);

        context
    }

    fn rsync_to_from<U: AsRef<str>, O: AsRef<OsStr>, S: AsRef<str>, D: AsRef<str>>(
        &self,
        to: bool,
        user: &Option<U>,
        rsync_options: &[O],
        sources: &[S],
        destination: &Option<D>,
        check: bool,
    ) -> Result<Option<i32>> {
        let mut ssh_cmd = vec!["ssh"];
        let ssh_key: String;

        let context = self.context();
        let sources = template::renders(&context, sources, "rsync sources")?;

        let self_ssh =
            self.ssh.as_ref().ok_or_else(|| Error::VMHasNoSsh(self.name.to_string()))?;

        ssh_cmd.extend(&self_ssh.options());

        let port = self_ssh.port().to_string();
        let port = if port == "random" { self.cache.load("port")? } else { port };
        ssh_cmd.extend(["-p", &port]);

        if self_ssh.has_key() {
            let keys = self_ssh.ensure_keys(&self.vml_directory.join("ssh"))?;
            if let Some(pvt_key) = keys.private() {
                ssh_key = pvt_key;
                ssh_cmd.extend(["-o", "IdentitiesOnly=yes"]);
                ssh_cmd.extend(["-i", &ssh_key]);
            }
        }

        let user_host = self_ssh.user_host(user);

        let mut rsync = Command::new("rsync");
        rsync.arg("-e").arg(ssh_cmd.join(" "));
        rsync.args(rsync_options);
        let sources_str = sources.join(", ");

        if to {
            rsync.args(sources);
            if let Some(destination) = destination {
                let destination = template::render(&context, destination, "rsync destination")?;
                rsync.arg(format!("{}:{}", user_host, destination));
            }
        } else {
            let sources = sources.join(" ");
            rsync.arg(format!("{}:{}", user_host, sources));
            if let Some(destination) = destination {
                let destination = template::render(&context, destination, "rsync destination")?;
                rsync.arg(destination);
            }
        }

        debug!("{:#?}", &rsync);
        let rc = rsync.spawn().context("failed to run executable rsync")?.wait()?.code();
        if check && rc != Some(0) {
            if to {
                bail!(Error::RsyncTo(sources_str, self.name.to_string()));
            } else {
                bail!(Error::RsyncFrom(sources_str, self.name.to_string()));
            }
        }

        Ok(rc)
    }

    pub fn rsync_to<U: AsRef<str>, O: AsRef<OsStr>, S: AsRef<str>, D: AsRef<str>>(
        &self,
        user: &Option<U>,
        rsync_options: &[O],
        sources: &[S],
        destination: &Option<D>,
        check: bool,
    ) -> Result<Option<i32>> {
        self.rsync_to_from(true, user, rsync_options, sources, destination, check)
    }

    pub fn rsync_to_template<U: AsRef<str>, O: AsRef<OsStr>, T: AsRef<str>, D: AsRef<str>>(
        &self,
        user: &Option<U>,
        rsync_options: &[O],
        template_str: T,
        destination: &Option<D>,
        check: bool,
    ) -> Result<Option<i32>> {
        let tmp_dir = tempfile::tempdir().expect("can't create tmp file");
        let tmp_name = tmp_dir.path().join(template_str.as_ref()).to_string_lossy().to_string();
        let sources = [tmp_name.as_str()];
        template::render_file(
            &self.context(),
            template_str.as_ref(),
            &tmp_name,
            "rsync_to_template",
        )?;
        self.rsync_to_from(true, user, rsync_options, &sources, destination, check)
    }

    pub fn rsync_from<U: AsRef<str>, O: AsRef<OsStr>, S: AsRef<str>, D: AsRef<str>>(
        &self,
        user: &Option<U>,
        rsync_options: &[O],
        sources: &[S],
        destination: &Option<D>,
        check: bool,
    ) -> Result<Option<i32>> {
        self.rsync_to_from(false, user, rsync_options, sources, destination, check)
    }

    pub fn monitor(&self) -> Result<()> {
        Command::new("socat")
            .arg("-,echo=0,icanon=0")
            .arg(format!("unix-connect:{}", &self.monitor.to_string_lossy()))
            .spawn()
            .context("failed to run executable socat")?
            .wait()?;

        Ok(())
    }

    pub fn monitor_command<S: AsRef<str>>(&self, command: S) -> Result<Option<String>> {
        let command = &format!("{}\n", command.as_ref());
        let reply = socket::reply(command.as_bytes(), &self.monitor)?;
        let reply =
            String::from_utf8(reply).context("bad utf8 symbols in reply from qemu monitor")?;
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
            bail!(Error::RemoveRunningVM(self.name));
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

    pub fn run_program_with_context<
        P: AsRef<Path>,
        H: AsRef<Path>,
        G: AsRef<str>,
        U: AsRef<str>,
    >(
        &self,
        program: P,
        user: &Option<U>,
        copy_host_dir: &Option<H>,
        guest_working_dir: &Option<G>,
    ) -> Result<Option<i32>> {
        let prog = program.as_ref();
        let prog_name = prog
            .file_name()
            .ok_or_else(|| Error::BadProgramFilename(prog.display().to_string()))?;
        let prog_name = prog_name.to_string_lossy();

        let mut sources = vec![prog.to_string_lossy()];
        if let Some(dir) = copy_host_dir {
            let dir = dir.as_ref();
            sources.push(dir.to_string_lossy())
        }
        self.rsync_to(user, &["--archive"], &sources, guest_working_dir, true)?;

        let ssh_options: [String; 0] = [];
        let ssh_flags: [String; 0] = [];
        let guest_program_path = if let Some(dir) = guest_working_dir {
            let dir = dir.as_ref();
            format!("{}/{}", dir, &prog_name)
        } else {
            format!("./{}", &prog_name)
        };
        self.ssh(user, &ssh_options, &ssh_flags, &Some(vec![guest_program_path]))
    }

    pub fn clean<P: AsRef<Path>>(&self, cleanup_program: P) -> Result<()> {
        let user: Option<&str> = Some("root");

        let copy_host_dir: Option<PathBuf> = None;
        let guest_working_dir = Some(".");
        self.run_program_with_context(cleanup_program, &user, &copy_host_dir, &guest_working_dir)?;

        Ok(())
    }

    pub fn store_disk<P: AsRef<Path>>(&self, to: P, force: bool) -> Result<()> {
        let to = to.as_ref();

        if self.has_pid() {
            bail!(Error::StoreRunningVM(self.name.to_string()));
        }

        if to.exists() && !force {
            bail!(Error::RewriteExistsPath(to.to_string_lossy().to_string()));
        }

        fs::copy(&self.disk, to)?;

        Ok(())
    }

    pub fn info(&self) -> BTreeMap<&'static str, String> {
        let mut info = BTreeMap::from([
            ("memory", self.memory.to_string()),
            ("name", self.name.to_string()),
            ("nproc", self.nproc.to_string()),
            ("openssh_config", self.openssh_config.display().to_string()),
        ]);

        if let Some(pid) = &self.pid {
            info.insert("pid", pid.to_string());
        }

        if let Some(image_name) = &self.image_name {
            info.insert("image", image_name.to_string());
        } else {
            info.insert("image", "".to_string());
        }

        if let Some(net) = &self.net {
            match net {
                Net::User => {
                    info.insert("network", "user".to_string());
                }
                Net::Tap { address, tap, .. } => {
                    info.insert("network", format!("tap:{}", &tap));
                    if let Some(address) = address {
                        info.insert("network_address", address.to_string());
                    }
                }
            }
        }

        if let Some(ssh) = &self.ssh {
            let mut options = "".to_string();
            info.insert("ssh_host", ssh.host().to_string());

            if let Ok(Some(port)) = self.get_ssh_port() {
                info.insert("ssh_port", port);
            }

            if let Ok(Some(key)) = self.get_ssh_private_key() {
                info.insert("ssh_key", key);
                options.push_str(" -o IdentitiesOnly=yes");
            }

            if let Some(user) = ssh.user() {
                info.insert("ssh_user", user);
            }

            options.push(' ');
            options.push_str(&ssh.options().join(" "));
            if !options.is_empty() {
                info.insert("ssh_options", options);
            }
        };

        info
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

fn image_size<S: AsRef<OsStr>>(image: S) -> Result<u64> {
    let mut qemu_img = Command::new("qemu-img");
    qemu_img.args(["info", "--output=json"]).arg(image.as_ref());
    let out = qemu_img.output()?;

    let out = String::from_utf8(out.stdout)
        .with_context(|| format!("bad utf8 symbols in {:?} command output", &qemu_img))?;

    let parsed = json::parse(&out)
        .with_context(|| format!("failed to parse `{:?}` command output as json", &qemu_img))?;

    if let Some(size) = parsed["virtual-size"].as_u64() {
        return Ok(size);
    }

    bail!(
        "failed to parse `{:?}` out\n\nCaused by:\n\t{}",
        &qemu_img,
        "can't read virtual-size as u64"
    )
}

fn try_resize<S: AsRef<OsStr>>(image: S, size: u64) -> Result<()> {
    let image = image.as_ref();
    let current_size = image_size(image)?;
    let image = image.to_os_string();

    if current_size < size {
        let out =
            run_fun!(qemu-img resize $image $size).context("failed to run executable qemu-img")?;
        info!("{}", out);
    }

    Ok(())
}
