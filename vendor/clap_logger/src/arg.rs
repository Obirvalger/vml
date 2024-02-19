/// # Get LogLevel Arg
/// returns a `clap::Arg`, which accepts the log level via CLI
///
/// ## Arguments
/// default_loglevel: `log::LevelFilter` which will become the loglevel when no one is defined by the user.
pub fn get_loglevel_arg<'help>(default_loglevel: log::LevelFilter) -> clap::Arg<'help> {
	clap::Arg::new("loglevel")
		.long("loglevel")
		.required(false)
		.default_value(default_loglevel.as_str())
		.help("Set the loglevel")
		.long_help("Set the loglevel. TRACE is the most verbose and OFF the least verbose")
		.possible_values(["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"])
}

/// # CLap LogLevel arg
/// Trait which adds the loglevel argument.
///
/// Intended for `clap::Command`
pub trait ClapLoglevelArg {
	fn add_loglevel_arg(self, default_loglevel: log::LevelFilter) -> Self;
}

impl ClapLoglevelArg for clap::Command<'_> {
	/// TODO Docstring
	fn add_loglevel_arg(self, default_loglevel: log::LevelFilter) -> Self {
		self.arg(get_loglevel_arg(default_loglevel))
	}
}
