use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;

use clap::ArgMatches;

use vml::cli;
use vml::config::Config;
use vml::files;
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

fn set_specifications(vmc: &mut VMsCreator, matches: &ArgMatches) {
    if let Some(name) = matches.value_of("NAME") {
        vmc.name(name);
    }

    if matches.is_present("names") {
        let names: Vec<&str> = matches.values_of("names").unwrap().collect();
        vmc.names(&names);
    }

    if matches.is_present("parents") {
        let parents: Vec<&str> = matches.values_of("parents").unwrap().collect();
        vmc.parents(&parents);
    }

    if matches.is_present("tags") {
        let tags: Vec<&str> = matches.values_of("tags").unwrap().collect();
        vmc.tags(&tags);
    }

    if matches.is_present("running") {
        vmc.with_pid(WithPid::Filter);
    }
}

fn main() -> Result<()> {
    files::install_all()?;

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
        Some(("images", images_matches)) => {
            let images_dir = config.images.directory;

            match images_matches.subcommand() {
                Some(("list", _)) => {
                    for image in vml::images::list(&images_dir)? {
                        println!("{}", image);
                    }
                }

                Some(("available", _)) => {
                    for image in vml::images::available(&images_dir)? {
                        println!("{}", image);
                    }
                }

                Some(("pull", pull_images_matches)) => {
                    let image = pull_images_matches.value_of("IMAGE").unwrap();

                    vml::images::pull(&images_dir, image)?;
                }

                _ => println!("Unexpected images command"),
            }
        }

        Some(("create", create_matches)) => {
            let names: Vec<&str> = if create_matches.is_present("names") {
                create_matches.values_of("names").unwrap().collect()
            } else if let Some(name) = create_matches.value_of("NAME") {
                vec![name]
            } else {
                vec![]
            };

            let image = create_matches.value_of("image");

            for name in names {
                vml::create_vm(&config, name, image)?;
            }
        }

        Some(("start", start_matches)) => {
            if start_matches.is_present("all") {
                vmc.all();
            }

            set_specifications(&mut vmc, start_matches);

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

            set_specifications(&mut vmc, stop_matches);

            let force = stop_matches.is_present("force");

            vmc.with_pid(WithPid::Filter);
            vmc.error_on_empty();

            for vm in vmc.create()? {
                vm.stop(force)?;
            }
        }

        Some(("ssh", ssh_matches)) => {
            set_specifications(&mut vmc, ssh_matches);

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
            set_specifications(&mut vmc, rsync_to_matches);

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
            set_specifications(&mut vmc, rsync_from_matches);

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

            set_specifications(&mut vmc, show_matches);

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

            set_specifications(&mut vmc, list_matches);

            list(
                &vmc,
                &config,
                list_matches.is_present("fold"),
                list_matches.is_present("unfold"),
            )?;
        }

        Some(("monitor", monitor_matches)) => {
            set_specifications(&mut vmc, monitor_matches);

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
            set_specifications(&mut vmc, rm_matches);

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
