use clap::{App, AppSettings, Arg, ArgGroup, ValueHint};
use clap_generate::generators::{Bash, Elvish, Fish, PowerShell, Zsh};
use clap_generate::{generate, Generator};
use std::io;

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
        .version("0.1.0")
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .global_setting(AppSettings::InferSubcommands)
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
            App::new("images")
                .about("command to work with vm images")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(App::new("list").about("list vm images").alias("ls")),
        )
        .subcommand(
            App::new("create")
                .about("create virtual machine")
                .arg(Arg::new("names").long("names").short('n').takes_value(true).multiple(true))
                .arg(Arg::new("image").long("image").short('i').takes_value(true)),
        )
        .subcommand(
            App::new("start")
                .about("start virtual machines")
                .arg(Arg::new("names").long("names").short('n').takes_value(true).multiple(true))
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
        )
        .subcommand(
            App::new("stop")
                .about("stop virtual machines")
                .arg(Arg::new("force").long("force").short('f'))
                .arg(Arg::new("names").long("names").short('n').takes_value(true).multiple(true))
                .arg(Arg::new("all").long("all").short('a'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(Arg::new("tags").long("tags").short('t').takes_value(true).multiple(true)),
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
                .arg(Arg::new("names").long("names").short('n').takes_value(true).multiple(true))
                .arg(
                    Arg::new("cmd")
                        .long("cmd")
                        .short('c')
                        .takes_value(true)
                        .multiple(true)
                        .value_hint(ValueHint::CommandString),
                )
                .arg(Arg::new("all").long("all").short('a'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(
                    Arg::new("user")
                        .long("user")
                        .short('u')
                        .takes_value(true)
                        .value_hint(ValueHint::Username),
                )
                .arg(Arg::new("tags").long("tags").short('t').takes_value(true).multiple(true)),
        )
        .subcommand(
            App::new("rsync-to")
                .about("rsync to a virtual machine, default destination is home")
                .arg(Arg::new("names").long("names").short('n').takes_value(true).multiple(true))
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
                .arg(
                    Arg::new("user")
                        .long("user")
                        .short('u')
                        .takes_value(true)
                        .value_hint(ValueHint::Username),
                )
                .arg(Arg::new("tags").long("tags").short('t').takes_value(true).multiple(true))
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
                        .multiple(true),
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
                .arg(Arg::new("names").long("names").short('n').takes_value(true).multiple(true))
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
                .arg(
                    Arg::new("user")
                        .long("user")
                        .short('u')
                        .takes_value(true)
                        .value_hint(ValueHint::Username),
                )
                .arg(Arg::new("tags").long("tags").short('t').takes_value(true).multiple(true))
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
                        .multiple(true)
                        .required(true),
                )
                .group(ArgGroup::new("action").args(&["destination", "list"])),
        )
        .subcommand(
            App::new("show")
                .about("show virtual machines")
                .arg(Arg::new("names").long("names").short('n').takes_value(true).multiple(true))
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
                .arg(Arg::new("names").long("names").short('n').takes_value(true).multiple(true))
                .arg(Arg::new("fold").long("fold").short('f'))
                .arg(Arg::new("unfold").long("unfold").short('u'))
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
                .group(ArgGroup::new("fold_group").args(&["fold", "unfold"]))
                .group(ArgGroup::new("vms").args(&["names", "all", "parents", "tags"])),
        )
        .subcommand(
            App::new("monitor")
                .about("acces to qemu monitor")
                .arg(Arg::new("command").long("command").short('c').takes_value(true))
                .arg(Arg::new("names").long("names").short('n').takes_value(true).multiple(true))
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
            App::new("rm")
                .about("remove virtual machines")
                .arg(Arg::new("force").long("force").short('f'))
                .arg(Arg::new("names").long("names").short('n').takes_value(true).multiple(true))
                .arg(Arg::new("all").long("all").short('a'))
                .arg(
                    Arg::new("parents")
                        .long("parents")
                        .short('p')
                        .takes_value(true)
                        .multiple(true),
                )
                .arg(Arg::new("tags").long("tags").short('t').takes_value(true).multiple(true)),
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
