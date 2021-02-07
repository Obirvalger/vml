use std::collections::HashSet;
use std::env;

use vml::cli;
use vml::config::Config;
use vml::Result;
use vml::{VMsCreator, WithPid};

fn main() -> Result<()> {
    let matches = cli::build_cli().get_matches();

    let config = Config::new()?;

    match matches.subcommand() {
        Some(("start", start_matches)) => {
            let mut vmc = VMsCreator::new(&config);
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
            let mut vmc = VMsCreator::new(&config);
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
                if force || vm.ssh(&Some("root"), &[], &Some(vec!["poweroff"])).is_err() {
                    vm.stop()?;
                }
            }
        }

        Some(("ssh", ssh_matches)) => {
            let mut vmc = VMsCreator::new(&config);

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

            let sources: Vec<&str> = rsync_to_matches.values_of("sources").unwrap().collect();

            let destination = if rsync_to_matches.is_present("list") {
                None
            } else {
                Some(rsync_to_matches.value_of("destination").unwrap_or("~"))
            };

            vmc.with_pid(WithPid::Error);
            vmc.error_on_empty();
            for vm in vmc.create()? {
                vm.rsync_to(&user, &rsync_options, &sources, &destination)?;
            }
        }

        Some(("rsync-from", rsync_from_matches)) => {
            let mut vmc = VMsCreator::new(&config);

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
            let mut vmc = VMsCreator::new(&config);
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
            let mut vmc = VMsCreator::new(&config);
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

            let fold = if config.list_fold {
                list_matches.is_present("fold") || !list_matches.is_present("unfold")
            } else {
                list_matches.is_present("fold") && !list_matches.is_present("unfold")
            };

            let mut names: HashSet<String> = HashSet::new();

            for vm in vmc.create()? {
                if fold {
                    names.insert(vm.ancestor());
                } else {
                    names.insert(vm.name.to_owned());
                }
            }

            for name in names {
                println!("{}", name);
            }
        }
        Some(("completion", completion_matches)) => {
            cli::completion(completion_matches.value_of("SHELL").unwrap())
        }
        _ => println!("Unexpected command"),
    }

    Ok(())
}
