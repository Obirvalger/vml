fn main() {
	use clap::{arg, Arg, Command};
	use clap_logger::{ClapInitLogger, ClapLoglevelArg};
	use log::LevelFilter;

	// Generate a clap command
	let m: clap::ArgMatches = Command::new("clap_command_test")
		.arg(arg!(-a --alpha "hello world!"))
		.arg(
			Arg::new("input")
				.short('i')
				.long("input")
				.takes_value(true)
				.required(false),
		)
		// add the loglevel argument
		.add_loglevel_arg(LevelFilter::Warn)
		.get_matches();

	m.init_logger().expect("Failed to initialize logger");
}
