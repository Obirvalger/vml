//! # Clap Logger
//! Simple [env_logger][env_logger] integration for [clap][clap].
//!
//! This create provides a simple way to allow the user to set the log level via a command line argument.
//! Its directly implemented in clap, so it feels very naturally.
//!
//! Please note this crate does not support `clap_derive` and won't support it in the near future or possibly never,
//! since Integrating with it is very hard to do.
//!
//! ## Features
//! * Command line argument to set loglevel
//! * Argument can be modified
//! * Optional: Loglevel via Environment variables
//! * directly embedded in `clap::Command` and `clap::ArgMatches`
//!
//! ## Status: Beta
//! ### Finished
//! * Feature complete (But Open for suggestions)
//! * no panics
//!
//! ### TODO
//! * Waiting for feedback
//! * more tests
//! * Complete documentation
//! * more examples
//!
//! ## Backlog
//! * Figure out if `clap_derive` support possible,
//!
//! ## Adding the Argument
//! ### Base Implementation:
//! ```rust
//! use clap::Command;
//! use log::LevelFilter;
//! use clap_logger::{ClapInitLogger, ClapLoglevelArg};
//!
//! // Generate a clap command
//! let m: clap::ArgMatches = Command::new("clap_command_test")
//!   // add loglevel argument
//!		.add_loglevel_arg(LevelFilter::Info)
//! 	.get_matches();
//! ```
//!
//! ## loglevel Arg manipulation
//! You can also get the [Arg][clap::Arg] directly in order to modify it before adding:`
//! ```rust
//! use clap::{arg, Arg, Command};
//! use log::LevelFilter;
//! use clap_logger::{ClapInitLogger, get_loglevel_arg};
//!
//! // Generate a clap command
//! let m: clap::ArgMatches = Command::new("clap_command_test")
//!   // add the add loglevel argument
//!  	.arg(get_loglevel_arg(LevelFilter::Info)
//! 		// Adding a short version
//! 		.short('l')
//!     // changing the long version of the argument just because I can
//! 		.long("custom-loglevel")
//! 		.default_value("INFO")
//!   )
//! 	.get_matches();
//! ```
//! Warning: Do NOT touch `.possible_values`, `.id` field of the argument or anything in that modifies the input.
//!
//! ## Initialising the logger
//! ### Base implementation:
//! ```rust
//! use clap::Command;
//! use log::LevelFilter;
//! use clap_logger::{ClapInitLogger, ClapLoglevelArg};
//!
//! let m: clap::ArgMatches = Command::new("clap_command_test")
//!   // add the loglevel argument
//!  	.add_loglevel_arg(LevelFilter::Info)
//! 	.get_matches();
//!
//! m.init_logger().expect("Failed to initialize logger");
//! ```
//!

mod arg;
mod init_logger;
#[cfg(test)]
mod tests;

pub use log::{debug, error, info, trace, warn, LevelFilter};

pub use crate::arg::{get_loglevel_arg, ClapLoglevelArg};
pub use crate::init_logger::ClapInitLogger;
