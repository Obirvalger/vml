use std::io;
use std::path::PathBuf;

use anyhow::{bail, Result};
use byte_unit::Byte;
use clap::{crate_version, value_parser, Arg, ArgEnum, ArgGroup, Command, ValueHint};
use clap_complete::{generate, Generator, Shell};

fn print_completions<G: Generator>(gen: G, app: &mut Command) {
    generate(gen, app, app.get_name().to_string(), &mut io::stdout());
}

pub fn completion(shell: &str) -> Result<()> {
    let mut app = build_cli();
    if let Ok(gen) = Shell::from_str(shell, true) {
        print_completions(gen, &mut app)
    } else {
        bail!("Unknown shell `{}` for completion", shell)
    }

    Ok(())
}

pub fn build_cli() -> clap::Command<'static> {
    Command::new("vml")
        .about("virtual machines manage utility")
        .version(crate_version!())
        .arg_required_else_help(true)
        .infer_subcommands(true)
        .infer_long_args(true)
        .arg(Arg::new("all-vms").long("all-vms").help("Specify all vms"))
        .arg(
            Arg::new("host")
                .long("host")
                .short('H')
                .takes_value(true)
                .help("Run vml command on host"),
        )
        .arg(
            Arg::new("vm-config")
                .long("vm-config")
                .help("Path to vm config replacement (use tera templates)")
                .value_hint(ValueHint::FilePath)
                .takes_value(true),
        )
        .arg(
            Arg::new("minimal-vm-config")
                .long("minimal-vm-config")
                .help("Replace vm config with minimal one"),
        )
        .group(ArgGroup::new("vm-config-group").args(&["vm-config", "minimal-vm-config"]))
        .subcommand(
            Command::new("image")
                .about("command to work with vm images")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(Command::new("available").about("list available to pull vm images"))
                .subcommand(
                    Command::new("remove")
                        .about("remove images")
                        .visible_alias("rm")
                        .arg(Arg::new("all").long("all").short('a'))
                        .arg(
                            Arg::new("images")
                                .long("images")
                                .short('i')
                                .takes_value(true)
                                .multiple_values(true),
                        )
                        .arg(Arg::new("IMAGE").takes_value(true)),
                )
                .subcommand(Command::new("list").about("list vm images").visible_alias("ls"))
                .subcommand(
                    Command::new("pull")
                        .about("pull vm images")
                        .arg(
                            Arg::new("available")
                                .long("available")
                                .short('a')
                                .help("pull all available images"),
                        )
                        .arg(Arg::new("IMAGES").takes_value(true).multiple_values(true))
                        .arg(
                            Arg::new("exists")
                                .long("exists")
                                .short('e')
                                .help("pull all existing in images directory images"),
                        )
                        .arg(
                            Arg::new("outdate")
                                .long("outdate")
                                .short('o')
                                .help("pull all outdate images"),
                        )
                        .group(
                            ArgGroup::new("specified_by")
                                .args(&["IMAGES", "available", "exists", "outdate"])
                                .required(true),
                        ),
                )
                .replace("update", &["pull", "--outdate"])
                .subcommand(
                    Command::new("store")
                        .about("store vm disk as image")
                        .arg(
                            Arg::new("image")
                                .help("stored image name (allow templates) [hyphenized vm name]")
                                .long("image")
                                .short('i')
                                .takes_value(true),
                        )
                        .arg(
                            Arg::new("force")
                                .help("rewrite existing image")
                                .long("force")
                                .short('f'),
                        )
                        .arg(Arg::new("NAME").takes_value(true).required(true))
                        .arg(
                            Arg::new("names")
                                .long("names")
                                .short('n')
                                .takes_value(true)
                                .multiple_values(true),
                        )
                        .arg(
                            Arg::new("parents")
                                .long("parents")
                                .short('p')
                                .takes_value(true)
                                .multiple_values(true),
                        )
                        .arg(
                            Arg::new("tags")
                                .long("tags")
                                .short('t')
                                .takes_value(true)
                                .multiple_values(true),
                        ),
                ),
        )
        .replace("images", &["image", "ls"])
        .subcommand(
            Command::new("create")
                .about("create virtual machine")
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("name-same-image")
                        .long("name-same-image")
                        .short('N')
                        .takes_value(true),
                )
                .arg(Arg::new("image").long("image").short('i').takes_value(true))
                .arg(Arg::new("nproc").long("nproc").takes_value(true))
                .arg(Arg::new("memory").long("memory").short('m').takes_value(true))
                .arg(
                    Arg::new("minimum-disk-size")
                        .long("minimum-disk-size")
                        .takes_value(true)
                        .validator(|s| Byte::from_str(s).map(|b| b.to_string())),
                )
                .arg(Arg::new("ssh-user").long("ssh-user").takes_value(true))
                .arg(Arg::new("ssh-key").long("ssh-key").takes_value(true))
                .arg(
                    Arg::new("properties")
                        .long("properties")
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("net-tap").long("net-tap").takes_value(true))
                .arg(
                    Arg::new("net-address")
                        .long("net-address")
                        .takes_value(true)
                        .requires("net-tap")
                        .conflicts_with_all(&["net-user", "net-none"]),
                )
                .arg(
                    Arg::new("net-gateway")
                        .long("net-gateway")
                        .takes_value(true)
                        .requires("net-tap")
                        .conflicts_with_all(&["net-user", "net-none"]),
                )
                .arg(
                    Arg::new("net-nameservers")
                        .long("net-nameservers")
                        .takes_value(true)
                        .multiple_values(true)
                        .requires("net-tap")
                        .conflicts_with_all(&["net-user", "net-none"]),
                )
                .arg(Arg::new("net-none").long("net-none"))
                .arg(Arg::new("net-user").long("net-user"))
                .arg(Arg::new("nic-model").long("nic-model").takes_value(true))
                .arg(
                    Arg::new("cloud-init-image")
                        .long("cloud-init-image")
                        .takes_value(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(Arg::new("cloud-init").long("cloud-init"))
                .arg(Arg::new("no-cloud-init").long("no-cloud-init"))
                .arg(
                    Arg::new("display-console").long("display-console").help("use qemu nographic"),
                )
                .arg(Arg::new("display-gtk").long("display-gtk"))
                .arg(Arg::new("display-none").long("display-none"))
                .arg(Arg::new("exists-fail").long("exists-fail"))
                .arg(Arg::new("exists-ignore").long("exists-ignore"))
                .arg(Arg::new("exists-replace").long("exists-replace"))
                .group(ArgGroup::new("net").args(&["net-tap", "net-none", "net-user"]))
                .group(ArgGroup::new("cloud-init-group").args(&["cloud-init", "no-cloud-init"]))
                .group(ArgGroup::new("display").args(&[
                    "display-gtk",
                    "display-none",
                    "display-console",
                ]))
                .group(ArgGroup::new("exists").args(&[
                    "exists-fail",
                    "exists-ignore",
                    "exists-replace",
                ]))
                .group(
                    ArgGroup::new("name_group")
                        .args(&["names", "name-same-image", "NAME"])
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("start")
                .about("start virtual machines")
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("ssh").long("ssh"))
                .arg(Arg::new("no-ssh").long("no-ssh"))
                .arg(Arg::new("wait-ssh").long("wait-ssh"))
                .arg(Arg::new("no-wait-ssh").long("no-wait-ssh"))
                .arg(Arg::new("cloud-init").long("cloud-init").short('c'))
                .arg(Arg::new("no-cloud-init").long("no-cloud-init"))
                .arg(Arg::new("running-fail").long("running-fail"))
                .arg(Arg::new("running-ignore").long("running-ignore"))
                .arg(Arg::new("running-restart").long("running-restart").alias("running-stop"))
                .arg(
                    Arg::new("drives")
                        .long("drives")
                        .short('d')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .short('t')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .group(ArgGroup::new("ssh-group").args(&["ssh", "no-ssh"]))
                .group(ArgGroup::new("wait-ssh-group").args(&["wait-ssh", "no-wait-ssh"]))
                .group(ArgGroup::new("running-group").args(&[
                    "running-fail",
                    "running-ignore",
                    "running-restart",
                ]))
                .group(ArgGroup::new("cloud-init-group").args(&["cloud-init", "no-cloud-init"])),
        )
        .subcommand(
            Command::new("run")
                .about("shortcut to create and start")
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("name-same-image")
                        .long("name-same-image")
                        .short('N')
                        .takes_value(true),
                )
                .arg(Arg::new("ssh").long("ssh"))
                .arg(Arg::new("no-ssh").long("no-ssh"))
                .arg(Arg::new("wait-ssh").long("wait-ssh"))
                .arg(Arg::new("no-wait-ssh").long("no-wait-ssh"))
                .arg(
                    Arg::new("cloud-init-image")
                        .long("cloud-init-image")
                        .takes_value(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(Arg::new("cloud-init").long("cloud-init").short('c'))
                .arg(Arg::new("no-cloud-init").long("no-cloud-init"))
                .arg(
                    Arg::new("drives")
                        .long("drives")
                        .short('d')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("image").long("image").short('i').takes_value(true))
                .arg(Arg::new("nproc").long("nproc").takes_value(true))
                .arg(Arg::new("memory").long("memory").short('m').takes_value(true))
                .arg(
                    Arg::new("minimum-disk-size")
                        .long("minimum-disk-size")
                        .takes_value(true)
                        .validator(|s| Byte::from_str(s).map(|b| b.to_string())),
                )
                .arg(Arg::new("ssh-user").long("ssh-user").takes_value(true))
                .arg(Arg::new("ssh-key").long("ssh-key").takes_value(true))
                .arg(
                    Arg::new("properties")
                        .long("properties")
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("net-tap").long("net-tap").takes_value(true))
                .arg(
                    Arg::new("net-address")
                        .long("net-address")
                        .takes_value(true)
                        .requires("net-tap")
                        .conflicts_with_all(&["net-user", "net-none"]),
                )
                .arg(
                    Arg::new("net-gateway")
                        .long("net-gateway")
                        .takes_value(true)
                        .requires("net-tap")
                        .conflicts_with_all(&["net-user", "net-none"]),
                )
                .arg(
                    Arg::new("net-nameservers")
                        .long("net-nameservers")
                        .takes_value(true)
                        .multiple_values(true)
                        .requires("net-tap")
                        .conflicts_with_all(&["net-user", "net-none"]),
                )
                .arg(Arg::new("running-fail").long("running-fail"))
                .arg(Arg::new("running-ignore").long("running-ignore"))
                .arg(Arg::new("running-restart").long("running-restart").alias("running-stop"))
                .arg(Arg::new("net-none").long("net-none"))
                .arg(Arg::new("net-user").long("net-user"))
                .arg(Arg::new("nic-model").long("nic-model").takes_value(true))
                .arg(Arg::new("display-console").long("display-console"))
                .arg(Arg::new("display-gtk").long("display-gtk"))
                .arg(Arg::new("display-none").long("display-none"))
                .arg(Arg::new("exists-fail").long("exists-fail"))
                .arg(Arg::new("exists-ignore").long("exists-ignore"))
                .arg(Arg::new("exists-replace").long("exists-replace"))
                .group(ArgGroup::new("net").args(&["net-tap", "net-none", "net-user"]))
                .group(ArgGroup::new("display").args(&[
                    "display-gtk",
                    "display-none",
                    "display-console",
                ]))
                .group(ArgGroup::new("exists").args(&[
                    "exists-fail",
                    "exists-ignore",
                    "exists-replace",
                ]))
                .group(ArgGroup::new("running-group").args(&[
                    "running-fail",
                    "running-ignore",
                    "running-restart",
                ]))
                .group(ArgGroup::new("ssh-group").args(&["ssh", "no-ssh"]))
                .group(ArgGroup::new("wait-ssh-group").args(&["wait-ssh", "no-wait-ssh"]))
                .group(ArgGroup::new("cloud-init-group").args(&["cloud-init", "no-cloud-init"]))
                .group(
                    ArgGroup::new("name_group")
                        .args(&["names", "name-same-image", "NAME"])
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("clean")
                .about("clean virtual machines")
                .arg(
                    Arg::new("program")
                        .long("program")
                        .value_parser(value_parser!(PathBuf))
                        .takes_value(true)
                        .help("cleanup program"),
                )
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("all").long("all").short('a'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .short('t')
                        .takes_value(true)
                        .multiple_values(true),
                ),
        )
        .subcommand(
            Command::new("stop")
                .about("stop virtual machines")
                .arg(Arg::new("force").long("force").short('f'))
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("all").long("all").short('a'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .short('t')
                        .takes_value(true)
                        .multiple_values(true),
                ),
        )
        .subcommand(
            Command::new("ssh")
                .about("ssh to a virtual machine")
                .arg(
                    Arg::new("ssh-options")
                        .long("ssh-options")
                        .takes_value(true)
                        .default_values(&[])
                        .multiple_values(true),
                )
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("rm").long("rm").help("remove vm after ssh"))
                .arg(Arg::new("A").short('A').help("pass -A to ssh command"))
                .arg(Arg::new("N").short('N').help("pass -N to ssh command"))
                .arg(Arg::new("Y").short('Y').help("pass -Y to ssh command"))
                .arg(Arg::new("f").short('f').help("pass -f to ssh command"))
                .arg(
                    Arg::new("L")
                        .short('L')
                        .takes_value(true)
                        .value_name("address")
                        .help("same as -L ssh option"),
                )
                .arg(
                    Arg::new("R")
                        .short('R')
                        .takes_value(true)
                        .value_name("address")
                        .help("same as -R ssh option"),
                )
                .arg(
                    Arg::new("W")
                        .short('W')
                        .takes_value(true)
                        .value_name("host_port")
                        .help("same as -W ssh option"),
                )
                .arg(
                    Arg::new("check")
                        .long("check")
                        .help("fail on first command with non zero return code"),
                )
                .arg(
                    Arg::new("cmd")
                        .long("cmd")
                        .short('c')
                        .takes_value(true)
                        .multiple_values(true)
                        .value_hint(ValueHint::CommandString),
                )
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("user")
                        .long("user")
                        .short('u')
                        .takes_value(true)
                        .value_hint(ValueHint::Username),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .short('t')
                        .takes_value(true)
                        .multiple_values(true),
                ),
        )
        .subcommand(
            Command::new("rsync-to")
                .about("rsync to a virtual machine, default destination is home")
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("rsync-options")
                        .long("rsync-options")
                        .takes_value(true)
                        .default_values(&[])
                        .allow_hyphen_values(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("archive")
                        .long("archive")
                        .short('a')
                        .help("pass --archive to rsync command"),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .help("pass --verbose to rsync command"),
                )
                .arg(Arg::new("check").long("check").help("fail on rsync error"))
                .arg(Arg::new("no-check").long("no-check").help("do not fail on rsync error"))
                .arg(Arg::new("P").short('P').help("pass -P to rsync command"))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("user")
                        .long("user")
                        .short('u')
                        .takes_value(true)
                        .value_hint(ValueHint::Username),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .short('t')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("destination")
                        .long("destination")
                        .takes_value(true)
                        .short('d')
                        .value_hint(ValueHint::AnyPath),
                )
                .arg(Arg::new("list").long("list").short('l'))
                .arg(
                    Arg::new("sources")
                        .long("sources")
                        .short('s')
                        .value_hint(ValueHint::AnyPath)
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("template")
                        .long("template")
                        .value_hint(ValueHint::AnyPath)
                        .takes_value(true),
                )
                .group(ArgGroup::new("check-group").args(&["check", "no-check"]))
                .group(ArgGroup::new("source").args(&["sources", "template"]).required(true))
                .group(ArgGroup::new("action").args(&["destination", "list"])),
        )
        .subcommand(
            Command::new("rsync-from")
                .about("rsync from a virtual machine, default destination is CWD")
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("rsync-options")
                        .long("rsync-options")
                        .takes_value(true)
                        .default_values(&[])
                        .allow_hyphen_values(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("archive")
                        .long("archive")
                        .short('a')
                        .help("pass --archive to rsync command"),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .help("pass --verbose to rsync command"),
                )
                .arg(Arg::new("P").short('P').help("pass -P to rsync command"))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("check").long("check").help("fail on rsync error"))
                .arg(Arg::new("no-check").long("no-check").help("do not fail on rsync error"))
                .arg(
                    Arg::new("user")
                        .long("user")
                        .short('u')
                        .takes_value(true)
                        .value_hint(ValueHint::Username),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .short('t')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("destination")
                        .long("destination")
                        .takes_value(true)
                        .short('d')
                        .value_hint(ValueHint::AnyPath),
                )
                .arg(Arg::new("list").long("list").short('l'))
                .arg(
                    Arg::new("sources")
                        .long("sources")
                        .short('s')
                        .value_hint(ValueHint::AnyPath)
                        .takes_value(true)
                        .multiple_values(true)
                        .required(true),
                )
                .group(ArgGroup::new("check-group").args(&["check", "no-check"]))
                .group(ArgGroup::new("action").args(&["destination", "list"])),
        )
        .subcommand(
            Command::new("show")
                .about("show virtual machines")
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("all").long("all").short('a'))
                .arg(Arg::new("format-debug").long("format-debug").short('d'))
                .arg(Arg::new("format-json").long("format-json").short('j'))
                .arg(Arg::new("format-table").long("format-table"))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .short('t')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("running").long("running").short('r'))
                .group(ArgGroup::new("format").args(&["format-debug", "format-json"])),
        )
        .subcommand(Command::new("scp").about("show how to use scp with vml vms"))
        .subcommand(
            Command::new("list")
                .about("list virtual machines")
                .visible_alias("ls")
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("fold").long("fold").short('f'))
                .arg(Arg::new("unfold").long("unfold").short('u'))
                .arg(Arg::new("all").long("all").short('a'))
                .arg(Arg::new("running").long("running").short('r'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .short('t')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .group(ArgGroup::new("all_running").args(&["all", "running"]))
                .group(ArgGroup::new("fold_group").args(&["fold", "unfold"])),
        )
        .subcommand(
            Command::new("monitor")
                .about("acces to qemu monitor")
                .arg(Arg::new("command").long("command").short('c').takes_value(true))
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .short('t')
                        .takes_value(true)
                        .multiple_values(true),
                ),
        )
        .subcommand(
            Command::new("remove")
                .about("remove virtual machines")
                .visible_alias("rm")
                .arg(Arg::new("force").long("force").short('f'))
                .arg(Arg::new("verbose").long("verbose").short('v'))
                .arg(Arg::new("interactive").long("interactive").short('i'))
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .short('t')
                        .takes_value(true)
                        .multiple_values(true),
                ),
        )
        .subcommand(
            Command::new("get-file")
                .about("show embedded file")
                .arg(Arg::new("path").required(true)),
        )
        .subcommand(Command::new("completion").arg(
            Arg::new("SHELL").help("generate completions").required(true).possible_values([
                "bash",
                "elvish",
                "fish",
                "powershell",
                "zsh",
            ]),
        ))
}
