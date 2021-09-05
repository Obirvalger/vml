use std::io;

use byte_unit::Byte;
use clap::{App, AppSettings, Arg, ArgGroup, ValueHint};
use clap_generate::generators::{Bash, Elvish, Fish, PowerShell, Zsh};
use clap_generate::{generate, Generator};

fn print_completions<G: Generator>(app: &mut App) {
    generate::<G, _>(app, app.get_name().to_string(), &mut io::stdout());
}

pub fn completion(shell: &str) {
    let mut app = build_cli();
    match shell {
        "bash" => print_completions::<Bash>(&mut app),
        "elvish" => print_completions::<Elvish>(&mut app),
        "fish" => print_completions::<Fish>(&mut app),
        "powershell" => print_completions::<PowerShell>(&mut app),
        "zsh" => print_completions::<Zsh>(&mut app),
        _ => panic!("Unknown generator"),
    }
}

pub fn build_cli() -> clap::App<'static> {
    App::new("vml")
        .about("virtual machines manage utility")
        .version("0.1.2")
        .setting(AppSettings::NoAutoVersion)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .global_setting(AppSettings::InferSubcommands)
        .arg(Arg::new("all-vms").long("all-vms").about("Specify all vms"))
        .arg(
            Arg::new("host")
                .long("host")
                .short('H')
                .takes_value(true)
                .about("Run vml command on host"),
        )
        .arg(
            Arg::new("vm-config")
                .long("vm-config")
                .about("Path to vm config replacement (use tera templates)")
                .value_hint(ValueHint::FilePath)
                .takes_value(true),
        )
        .arg(
            Arg::new("minimal-vm-config")
                .long("minimal-vm-config")
                .about("Replace vm config with minimal one"),
        )
        .group(ArgGroup::new("vm-config-group").args(&["vm-config", "minimal-vm-config"]))
        .subcommand(
            App::new("image")
                .about("command to work with vm images")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(App::new("available").about("list available to pull vm images"))
                .subcommand(
                    App::new("remove")
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
                .subcommand(App::new("list").about("list vm images").visible_alias("ls"))
                .subcommand(
                    App::new("pull")
                        .about("pull vm images")
                        .arg(
                            Arg::new("available")
                                .long("available")
                                .short('a')
                                .about("pull all available images"),
                        )
                        .arg(Arg::new("IMAGES").takes_value(true).multiple_values(true))
                        .arg(
                            Arg::new("exists")
                                .long("exists")
                                .short('e')
                                .about("pull all existing in images directory images"),
                        )
                        .group(
                            ArgGroup::new("specified_by")
                                .args(&["IMAGES", "available", "exists"])
                                .required(true),
                        ),
                )
                .subcommand(
                    App::new("store")
                        .about("store vm disk as image")
                        .arg(
                            Arg::new("image")
                                .about("stored image name (allow templates) [hyphenized vm name]")
                                .long("image")
                                .short('i')
                                .takes_value(true),
                        )
                        .arg(
                            Arg::new("force")
                                .about("rewrite existing image")
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
            App::new("create")
                .about("create virtual machine")
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("image").long("image").short('i').takes_value(true))
                .arg(Arg::new("memory").long("memory").short('m').takes_value(true))
                .arg(
                    Arg::new("minimum-disk-size")
                        .long("minimum-disk-size")
                        .takes_value(true)
                        .validator(|s| Byte::from_str(s).map(|b| b.to_string())),
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
                .arg(Arg::new("cloud-init").long("cloud-init"))
                .arg(Arg::new("no-cloud-init").long("no-cloud-init"))
                .arg(Arg::new("display-gtk").long("display-gtk"))
                .arg(Arg::new("display-none").long("display-none"))
                .arg(Arg::new("exists-fail").long("exists-fail"))
                .arg(Arg::new("exists-ignore").long("exists-ignore"))
                .arg(Arg::new("exists-replace").long("exists-replace"))
                .group(ArgGroup::new("net").args(&["net-tap", "net-none", "net-user"]))
                .group(ArgGroup::new("cloud-init-group").args(&["cloud-init", "no-cloud-init"]))
                .group(ArgGroup::new("display").args(&["display-gtk", "display-none"]))
                .group(ArgGroup::new("exists").args(&[
                    "exists-fail",
                    "exists-ignore",
                    "exists-replace",
                ]))
                .group(ArgGroup::new("name_group").args(&["names", "NAME"]).required(true)),
        )
        .subcommand(
            App::new("start")
                .about("start virtual machines")
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("wait-ssh").long("wait-ssh"))
                .arg(Arg::new("no-wait-ssh").long("no-wait-ssh"))
                .arg(Arg::new("cloud-init").long("cloud-init").short('c'))
                .arg(Arg::new("no-cloud-init").long("no-cloud-init"))
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
                .group(ArgGroup::new("cloud-init-group").args(&["cloud-init", "no-cloud-init"])),
        )
        .subcommand(
            App::new("run")
                .about("shortcut to create and start")
                .arg(Arg::new("NAME").takes_value(true))
                .arg(
                    Arg::new("names")
                        .long("names")
                        .short('n')
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(Arg::new("wait-ssh").long("wait-ssh"))
                .arg(Arg::new("no-wait-ssh").long("no-wait-ssh"))
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
                .arg(Arg::new("memory").long("memory").short('m').takes_value(true))
                .arg(
                    Arg::new("minimum-disk-size")
                        .long("minimum-disk-size")
                        .takes_value(true)
                        .validator(|s| Byte::from_str(s).map(|b| b.to_string())),
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
                .arg(Arg::new("display-gtk").long("display-gtk"))
                .arg(Arg::new("display-none").long("display-none"))
                .arg(Arg::new("exists-fail").long("exists-fail"))
                .arg(Arg::new("exists-ignore").long("exists-ignore"))
                .arg(Arg::new("exists-replace").long("exists-replace"))
                .group(ArgGroup::new("net").args(&["net-tap", "net-none", "net-user"]))
                .group(ArgGroup::new("display").args(&["display-gtk", "display-none"]))
                .group(ArgGroup::new("exists").args(&[
                    "exists-fail",
                    "exists-ignore",
                    "exists-replace",
                ]))
                .group(ArgGroup::new("cloud-init-group").args(&["cloud-init", "no-cloud-init"]))
                .group(ArgGroup::new("name_group").args(&["names", "NAME"]).required(true)),
        )
        .subcommand(
            App::new("stop")
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
            App::new("ssh")
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
                .arg(Arg::new("A").short('A').about("pass -A to ssh command"))
                .arg(Arg::new("N").short('N').about("pass -N to ssh command"))
                .arg(Arg::new("Y").short('Y').about("pass -Y to ssh command"))
                .arg(Arg::new("f").short('f').about("pass -f to ssh command"))
                .arg(
                    Arg::new("L")
                        .short('L')
                        .takes_value(true)
                        .value_name("address")
                        .about("same as -L ssh option"),
                )
                .arg(
                    Arg::new("R")
                        .short('R')
                        .takes_value(true)
                        .value_name("address")
                        .about("same as -R ssh option"),
                )
                .arg(
                    Arg::new("check")
                        .long("check")
                        .about("fail on first command with non zero return code"),
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
            App::new("rsync-to")
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
                        .about("pass --archive to rsync command"),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .about("pass --verbose to rsync command"),
                )
                .arg(Arg::new("P").short('P').about("pass -P to rsync command"))
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
                .group(ArgGroup::new("source").args(&["sources", "template"]).required(true))
                .group(ArgGroup::new("action").args(&["destination", "list"])),
        )
        .subcommand(
            App::new("rsync-from")
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
                        .about("pass --archive to rsync command"),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .about("pass --verbose to rsync command"),
                )
                .arg(Arg::new("P").short('P').about("pass -P to rsync command"))
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
                        .multiple_values(true)
                        .required(true),
                )
                .group(ArgGroup::new("action").args(&["destination", "list"])),
        )
        .subcommand(
            App::new("show")
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
                .group(ArgGroup::new("all_running").args(&["all", "running"])),
        )
        .subcommand(
            App::new("list")
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
            App::new("monitor")
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
            App::new("remove")
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
        .subcommand(App::new("completion").arg(
            Arg::new("SHELL").about("generate completions").required(true).possible_values(&[
                "bash",
                "elvish",
                "fish",
                "powershell",
                "zsh",
            ]),
        ))
}
