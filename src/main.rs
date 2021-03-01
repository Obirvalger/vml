use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;

use vml::cli;
use vml::config::Config;
use vml::Result;
use vml::{VMsCreator, WithPid};

fn list(vmc: &VMsCreator, config: &Config, fold: bool, unfold: bool) -> Result<()> {
    let fold = if config.list_fold { fold || !unfold } else { fold && !unfold };

    let mut names: BTreeSet<String> = BTreeSet::new();

    for vm in vmc.create()? {
        if fold {
            names.insert(vm.folded_name());
        } else {
            names.insert(vm.name.to_owned());
        }
    }

    for name in names {
        println!("{}", name);
    }

    Ok(())
}

fn confirm(message: &str) -> bool {
    println!("{}", message);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Cannot read from stdin");
    let input = input.trim_end();
    let input = input.to_lowercase();

    matches!(input.as_str(), "y" | "yes")
}

fn main() -> Result<()> {
    let matches = cli::build_cli().get_matches();

    let config = Config::new()?;
    let mut vmc = VMsCreator::new(&config);
    if let Some(vm_config) = matches.value_of("vm-config") {
        vmc.vm_config(&fs::read_to_string(&vm_config)?);
    }
    if matches.is_present("minimal-vm-config") {
        vmc.minimal_vm_config();
    }

    match matches.subcommand() {
        Some(("create", create_matches)) => {
            let names: Vec<&str> = create_matches.values_of("names").unwrap().collect();

            let image = create_matches.value_of("image");

            for name in names {
                vml::create_vm(&config, name, image)?;
            }
        }

        Some(("start", start_matches)) => {
            if start_matches.is_present("all") {
                vmc.all();
            }

            if start_matches.is_present("names") {
                let names: Vec<&str> = start_matches.values_of("names").unwrap().collect();
                vmc.names(&names);
            }

            if start_matches.is_present("parents") {
                let parents: Vec<&str> = start_matches.values_of("parents").unwrap().collect();
                vmc.parents(&parents);
            }

            if start_matches.is_present("tags") {
                let tags: Vec<&str> = start_matches.values_of("tags").unwrap().collect();
                vmc.tags(&tags);
            }

            let cloud_init = start_matches.is_present("cloud-init");

            vmc.with_pid(WithPid::Without);
            vmc.error_on_empty();

            for vm in vmc.create()? {
                vm.start(cloud_init)?;
            }
        }

        Some(("stop", stop_matches)) => {
            if stop_matches.is_present("all") {
                vmc.all();
            }

            if stop_matches.is_present("names") {
                let names: Vec<&str> = stop_matches.values_of("names").unwrap().collect();
                vmc.names(&names);
            }

            if stop_matches.is_present("parents") {
                let parents: Vec<&str> = stop_matches.values_of("parents").unwrap().collect();
                vmc.parents(&parents);
            }

            if stop_matches.is_present("tags") {
                let tags: Vec<&str> = stop_matches.values_of("tags").unwrap().collect();
                vmc.tags(&tags);
            }

            let force = stop_matches.is_present("force");

            vmc.with_pid(WithPid::Filter);
            vmc.error_on_empty();

            for vm in vmc.create()? {
                vm.stop(force)?;
            }
        }

        Some(("ssh", ssh_matches)) => {
            if ssh_matches.is_present("parents") {
                let parents: Vec<&str> = ssh_matches.values_of("parents").unwrap().collect();
                vmc.parents(&parents);
            }

            if ssh_matches.is_present("tags") {
                let tags: Vec<&str> = ssh_matches.values_of("tags").unwrap().collect();
                vmc.tags(&tags);
            }

            if ssh_matches.is_present("names") {
                let names: Vec<&str> = ssh_matches.values_of("names").unwrap().collect();
                vmc.names(&names);
            }

            let user = ssh_matches.value_of("user");

            let ssh_options: Vec<&str> = if ssh_matches.is_present("ssh-options") {
                ssh_matches.values_of("ssh-options").unwrap().collect()
            } else {
                Vec::new()
            };

            let cmd: Option<Vec<&str>> = ssh_matches.values_of("cmd").map(|v| v.collect());

            vmc.with_pid(WithPid::Error);
            vmc.error_on_empty();
            for vm in vmc.create()? {
                vm.ssh(&user, &ssh_options, &cmd)?;
            }
        }

        Some(("rsync-to", rsync_to_matches)) => {
            let mut vmc = VMsCreator::new(&config);

            if rsync_to_matches.is_present("parents") {
                let parents: Vec<&str> = rsync_to_matches.values_of("parents").unwrap().collect();
                vmc.parents(&parents);
            }

            if rsync_to_matches.is_present("tags") {
                let tags: Vec<&str> = rsync_to_matches.values_of("tags").unwrap().collect();
                vmc.tags(&tags);
            }

            if rsync_to_matches.is_present("names") {
                let names: Vec<&str> = rsync_to_matches.values_of("names").unwrap().collect();
                vmc.names(&names);
            }

            let user = rsync_to_matches.value_of("user");

            let rsync_options: Vec<&str> = if rsync_to_matches.is_present("rsync-options") {
                rsync_to_matches.values_of("rsync-options").unwrap().collect()
            } else {
                Vec::new()
            };

            let sources = rsync_to_matches.values_of("sources");
            let template = rsync_to_matches.value_of("template");

            let destination = if rsync_to_matches.is_present("list") {
                None
            } else {
                Some(rsync_to_matches.value_of("destination").unwrap_or("~"))
            };

            vmc.with_pid(WithPid::Error);
            vmc.error_on_empty();
            if let Some(sources) = sources {
                let sources: Vec<&str> = sources.collect();
                for vm in vmc.create()? {
                    vm.rsync_to(&user, &rsync_options, &sources, &destination)?;
                }
            } else if let Some(template) = template {
                for vm in vmc.create()? {
                    vm.rsync_to_template(&user, &rsync_options, template, &destination)?;
                }
            }
        }

        Some(("rsync-from", rsync_from_matches)) => {
            if rsync_from_matches.is_present("parents") {
                let parents: Vec<&str> =
                    rsync_from_matches.values_of("parents").unwrap().collect();
                vmc.parents(&parents);
            }

            if rsync_from_matches.is_present("tags") {
                let tags: Vec<&str> = rsync_from_matches.values_of("tags").unwrap().collect();
                vmc.tags(&tags);
            }

            if rsync_from_matches.is_present("names") {
                let names: Vec<&str> = rsync_from_matches.values_of("names").unwrap().collect();
                vmc.names(&names);
            }

            let user = rsync_from_matches.value_of("user");

            let rsync_options: Vec<&str> = if rsync_from_matches.is_present("rsync-options") {
                rsync_from_matches.values_of("rsync-options").unwrap().collect()
            } else {
                Vec::new()
            };

            let sources: Vec<&str> = rsync_from_matches.values_of("sources").unwrap().collect();

            let cwd = env::current_dir().unwrap();
            let cwd = cwd.to_string_lossy();
            let destination = if rsync_from_matches.is_present("list") {
                None
            } else {
                Some(rsync_from_matches.value_of("destination").unwrap_or(&cwd))
            };

            vmc.with_pid(WithPid::Error);
            vmc.error_on_empty();
            for vm in vmc.create()? {
                vm.rsync_from(&user, &rsync_options, &sources, &destination)?;
            }
        }

        Some(("show", show_matches)) => {
            vmc.all();

            if show_matches.is_present("names") {
                let names: Vec<&str> = show_matches.values_of("names").unwrap().collect();
                vmc.names(&names);
            }

            if show_matches.is_present("parents") {
                let parents: Vec<&str> = show_matches.values_of("parents").unwrap().collect();
                vmc.parents(&parents);
            }

            if show_matches.is_present("tags") {
                let tags: Vec<&str> = show_matches.values_of("tags").unwrap().collect();
                vmc.tags(&tags);
            }

            if show_matches.is_present("running") {
                vmc.with_pid(WithPid::Filter);
            } else {
                vmc.with_pid(WithPid::Option);
            }
            vmc.error_on_empty();

            for vm in vmc.create()? {
                println!("{:#?}", vm);
            }
        }

        Some(("list", list_matches)) => {
            vmc.all();

            if list_matches.is_present("parents") {
                let parents: Vec<&str> = list_matches.values_of("parents").unwrap().collect();
                vmc.parents(&parents);
            }

            if list_matches.is_present("tags") {
                let tags: Vec<&str> = list_matches.values_of("tags").unwrap().collect();
                vmc.tags(&tags);
            }

            if list_matches.is_present("names") {
                let names: Vec<&str> = list_matches.values_of("names").unwrap().collect();
                vmc.names(&names);
            }

            if list_matches.is_present("running") {
                vmc.with_pid(WithPid::Filter);
            }

            list(
                &vmc,
                &config,
                list_matches.is_present("fold"),
                list_matches.is_present("unfold"),
            )?;
        }

        Some(("monitor", monitor_matches)) => {
            if monitor_matches.is_present("parents") {
                let parents: Vec<&str> = monitor_matches.values_of("parents").unwrap().collect();
                vmc.parents(&parents);
            }

            if monitor_matches.is_present("tags") {
                let tags: Vec<&str> = monitor_matches.values_of("tags").unwrap().collect();
                vmc.tags(&tags);
            }

            if monitor_matches.is_present("names") {
                let names: Vec<&str> = monitor_matches.values_of("names").unwrap().collect();
                vmc.names(&names);
            }

            if monitor_matches.is_present("running") {
                vmc.with_pid(WithPid::Filter);
            }

            let command = monitor_matches.value_of("command");

            vmc.with_pid(WithPid::Error);
            vmc.error_on_empty();
            if let Some(command) = command {
                for vm in vmc.create()? {
                    let reply = vm.monitor_command(command)?;
                    if let Some(reply) = reply {
                        println!("{}", reply);
                    }
                }
            } else {
                for vm in vmc.create()? {
                    vm.monitor()?;
                }
            }
        }

        Some(("rm", rm_matches)) => {
            if rm_matches.is_present("parents") {
                let parents: Vec<&str> = rm_matches.values_of("parents").unwrap().collect();
                vmc.parents(&parents);
            }

            if rm_matches.is_present("tags") {
                let tags: Vec<&str> = rm_matches.values_of("tags").unwrap().collect();
                vmc.tags(&tags);
            }

            if rm_matches.is_present("names") {
                let names: Vec<&str> = rm_matches.values_of("names").unwrap().collect();
                vmc.names(&names);
            }

            let force = rm_matches.is_present("force");

            if !force {
                vmc.with_pid(WithPid::Without);
            }

            list(&vmc, &config, false, false)?;
            let remove = confirm("Do you really want to remove that vms?");

            if remove {
                for vm in vmc.create()? {
                    if force && vm.has_pid() {
                        vm.stop(true)?;
                    }

                    vm.remove()?;
                }
            }
        }

        Some(("completion", completion_matches)) => {
            cli::completion(completion_matches.value_of("SHELL").unwrap())
        }

        _ => println!("Unexpected command"),
    }

    Ok(())
}
