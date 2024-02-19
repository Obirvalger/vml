#[cfg(feature = "env")]
use std::env::var as env_var;
use std::error::Error as StdError;
use std::fmt::Display;
use std::str::FromStr;

/// # Clap Logger Result
/// Result returning defined type or error
pub type CLapLoggerResult<T> = Result<T, Error>;

/// # Init Logger Error
/// Errors occurring in `clap_logger::init_logger`
#[derive(Debug)]
pub enum Error {
	/// # No Log Level was specified
	///
	/// If you get this error and haven't messed with the loglevel arg id, please report.
	NoLogLevelSupplied,
	/// # The log level could not be parsed
	/// This means that the loglevel supplied is not available.
	///
	/// This error should not occur, since clap only accepts the defined possible_values.
	///
	/// If you get this error and didn't mess with the loglevel argument input, id , please open an Issue.
	CouldNotParseLogLevel(log::ParseLevelError),
}

/// Define what to do with the log level from the environment if one is specified.
#[cfg(feature = "env")]
pub enum EnvLogLevelHandling {
	/// # Overwrite the default_loglevel
	/// If user specified a loglevel, use the user specified one, otherwise use the one from the environment.
	/// ## Input
	/// Name of the environment variable the loglevel is read from
	///
	/// ## Error Handling
	/// When loglevel from the environment variable is invalid, the default loglevel will be used
	OverwriteDefault(String),
	/// # Overwrite the loglevel supplied via the option.
	///
	/// ## Input
	/// Name of the environment variable the loglevel is read from
	///
	/// ## Error Handling
	/// When loglevel from the environment variable is invalid, the default/user-specified loglevel will be used.
	///
	OverwriteArgument(String),
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::NoLogLevelSupplied => write!(f, "No loglevel specified. Please Report."),
			Error::CouldNotParseLogLevel(e) => write!(f, "Failed to parse loglevel: {}", e),
		}
	}
}

impl StdError for Error {}

#[cfg(feature = "env")]
pub enum PrintEnvWarning {
	Yes,
	No,
}

/// # Clap Init Logger
/// Trait TODO docs
pub trait ClapInitLogger {
	fn init_logger(self) -> CLapLoggerResult<Self>
	where
		Self: Sized;

	fn get_loglevel(&self) -> CLapLoggerResult<log::LevelFilter>;

	#[cfg(feature = "env")]
	fn init_logger_env(
		self,
		env_loglevel_handling: EnvLogLevelHandling,
		print_hint: PrintEnvWarning,
	) -> CLapLoggerResult<Self>
	where
		Self: Sized;

	#[cfg(feature = "env")]
	fn get_loglevel_env(
		&self,
		env_loglevel_handling: EnvLogLevelHandling,
		print_env_warning: PrintEnvWarning,
	) -> CLapLoggerResult<log::LevelFilter>;
}

impl ClapInitLogger for clap::ArgMatches {
	// TODO doc
	fn init_logger(self) -> CLapLoggerResult<Self> {
		env_logger::builder()
			.filter_level(self.get_loglevel()?)
			.init();
		Ok(self)
	}

	// TODO Doc
	fn get_loglevel(&self) -> CLapLoggerResult<log::LevelFilter> {
		let levelfilter = match self.value_of("loglevel") {
			Some(r) => r,
			None => return Err(Error::NoLogLevelSupplied),
		};

		match log::LevelFilter::from_str(levelfilter) {
			Ok(r) => Ok(r),
			Err(e) => Err(Error::CouldNotParseLogLevel(e)), // Err
		}
	} // fn

	#[cfg(feature = "env")]
	fn init_logger_env(
		self,
		env_loglevel_handling: EnvLogLevelHandling,
		print_hint: PrintEnvWarning,
	) -> CLapLoggerResult<Self> {
		env_logger::builder()
			.filter_level(self.get_loglevel(env_loglevel_handling, print_hint)?)
			.init();
		Ok(self)
	} // fn

	#[cfg(feature = "env")]
	fn get_loglevel_env(
		&self,
		env_loglevel_handling: EnvLogLevelHandling,
		print_env_warning: PrintEnvWarning,
	) -> CLapLoggerResult<log::LevelFilter> {
		let loglevel_set: bool = self.occurrences_of("loglevel") > 0;

		let loglevel = match log::LevelFilter::from_str(match self.value_of("loglevel") {
			Some(r) => r,
			None => return Err(Error::NoLogLevelSupplied),
		}) {
			Ok(r) => r,
			Err(e) => {
				return Err(Error::CouldNotParseLogLevel(e));
			} // Err
		}; // let match

		////////////////////////////////
		// Parse environment Loglevel //
		////////////////////////////////

		fn print_err(key: &str, value: &str) {
			if print_env_warning == PrintEnvWarning::Yes {
				println!(
					"Invalid Loglevel: {}={}. Using default loglevel ...",
					key, value
				);
			}
		}

		match env_loglevel_handling {
			EnvLogLevelHandling::Ignore => Ok(loglevel),
			EnvLogLevelHandling::OverwriteDefault(e) => {
				if loglevel_set {
					Ok(loglevel)
				}

				let env: String = env_var(e.as_str()).unwrap_or("".to_string());

				log::LevelFilter::from_str(&env).unwrap_or_else({
					print_err(&e, &env);
					loglevel
				})
			} // OverwriteDefault

			EnvLogLevelHandling::OverwriteArgument(e) => {
				let env: String = env_var(e.as_str()).unwrap_or("".to_string());

				log::LevelFilter::from_str(&env).unwrap_or_else({
					print_err(&e, &env);
					loglevel
				})
			} // OverwriteArgument
		} // match
	} // fn
}
