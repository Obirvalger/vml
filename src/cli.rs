use clap::{App, AppSettings, Arg, ArgGroup};

pub fn build_cli() -> clap::App<'static> {
    App::new("vml")
        .about("virtual machines manage utility")
        .version("0.1.0")
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .global_setting(AppSettings::InferSubcommands)
        .subcommand(
            App::new("start")
                .about("start virtual machines")
                .arg(Arg::new("NAMES").takes_value(true).multiple(true))
                .arg(Arg::new("cloud-init").long("cloud-init").short('c'))
                .arg(Arg::new("all").long("all").short('a'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(Arg::new("tags").long("tags").short('t').takes_value(true).multiple(true)),
            // .group(ArgGroup::new("vms").required(true).args(&["NAMES", "all", "tags"])),
        )
        .subcommand(
            App::new("stop")
                .about("stop virtual machines")
                .arg(Arg::new("force").long("force").short('f'))
                .arg(Arg::new("NAMES").takes_value(true).multiple(true))
                .arg(Arg::new("all").long("all").short('a'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(Arg::new("tags").long("tags").short('t').takes_value(true).multiple(true)),
            // .group(ArgGroup::new("vms").required(true).args(&["NAMES", "all", "tags"])),
        )
        .subcommand(
            App::new("ssh")
                .about("ssh to a virtual machine")
                .arg(
                    Arg::new("ssh-options")
                        .long("ssh-options")
                        .takes_value(true)
                        .default_values(&[])
                        .allow_hyphen_values(true)
                        .multiple(true),
                )
                .arg(Arg::new("NAMES").takes_value(true).multiple(true))
                .arg(Arg::new("cmd").long("cmd").short('c').takes_value(true).multiple(true))
                .arg(Arg::new("all").long("all").short('a'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(Arg::new("user").long("user").short('u').takes_value(true))
                .arg(Arg::new("tags").long("tags").short('t').takes_value(true).multiple(true)),
        )
        .subcommand(
            App::new("rsync-to")
                .about("rsync to a virtual machine, default destination is home")
                .arg(Arg::new("NAMES").takes_value(true).multiple(true))
                .arg(
                    Arg::new("rsync-options")
                        .long("rsync-options")
                        .takes_value(true)
                        .default_values(&[])
                        .allow_hyphen_values(true)
                        .multiple(true),
                )
                .arg(Arg::new("all").long("all").short('a'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(Arg::new("user").long("user").short('u').takes_value(true))
                .arg(Arg::new("tags").long("tags").short('t').takes_value(true).multiple(true))
                .arg(Arg::new("destination").long("destination").takes_value(true).short('d'))
                .arg(Arg::new("list").long("list").short('l'))
                .arg(
                    Arg::new("sources")
                        .long("sources")
                        .short('s')
                        .takes_value(true)
                        .multiple(true)
                        .required(true),
                )
                .group(ArgGroup::new("action").args(&["destination", "list"]))
                .group(ArgGroup::new("all_running").args(&["all", "running"])),
        )
        .subcommand(
            App::new("rsync-from")
                .about("rsync from a virtual machine, default destination is CWD")
                .arg(Arg::new("NAMES").takes_value(true).multiple(true))
                .arg(
                    Arg::new("rsync-options")
                        .long("rsync-options")
                        .takes_value(true)
                        .default_values(&[])
                        .allow_hyphen_values(true)
                        .multiple(true),
                )
                .arg(Arg::new("all").long("all").short('a'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(Arg::new("user").long("user").short('u').takes_value(true))
                .arg(Arg::new("tags").long("tags").short('t').takes_value(true).multiple(true))
                .arg(Arg::new("destination").long("destination").takes_value(true).short('d'))
                .arg(Arg::new("list").long("list").short('l'))
                .arg(
                    Arg::new("sources")
                        .long("sources")
                        .short('s')
                        .takes_value(true)
                        .multiple(true)
                        .required(true),
                )
                .group(ArgGroup::new("action").args(&["destination", "list"]))
                .group(ArgGroup::new("all_running").args(&["all", "running"])),
        )
        .subcommand(
            App::new("show")
                .about("show virtual machines")
                .arg(Arg::new("NAMES").takes_value(true).multiple(true))
                .arg(Arg::new("all").long("all").short('a'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(Arg::new("tags").long("tags").short('t').takes_value(true).multiple(true))
                .arg(Arg::new("running").long("running").short('r'))
                .group(ArgGroup::new("all_running").args(&["all", "running"])),
        )
        .subcommand(
            App::new("list")
                .about("list virtual machines")
                .alias("ls")
                .arg(Arg::new("NAMES").takes_value(true).multiple(true))
                .arg(Arg::new("all").long("all").short('a'))
                .arg(Arg::new("running").long("running").short('r'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(Arg::new("tags").long("tags").short('t').takes_value(true).multiple(true))
                .group(ArgGroup::new("all_running").args(&["all", "running"]))
                .group(ArgGroup::new("vms").args(&["NAMES", "all", "parents", "tags"])),
        )
}
