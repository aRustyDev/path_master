mod paths;

use crate::paths::{PATHSD, ROOTD, get_paths_dirs_regex};
use clap::CommandFactory;
use clap::{Parser, Subcommand, ValueHint};
use clap_complete::{Shell, generate};
use std::io;
use std::path::Path;
use std::str::FromStr;

const ABOUT: &str = "A versatile 'helper' for constructing PATH like environment variables.";
const LONG_ABOUT: &str = "
The path_master utility reads the contents of the files in the directories /etc/*paths.d and appends their contents to either the PATH (default) or the <*>PATH environment variables respectively.
(Unlike path_helper the path_master will set ALL environment variables for any files in matching directories.)
";

#[derive(Parser, Debug)]
#[command(author, version, about = ABOUT, long_about = LONG_ABOUT)]
struct PathMaster {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Enable debugging
    #[arg(long)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// generate shell completions
    ///
    /// This command will output shell completion script
    /// to stdout. The kind of script is dependent on the
    /// argument passed to this subcommand.
    ///
    /// Enable (Current session):
    /// - Bash: `source <(path_master completion bash)`
    /// - Zsh: `source <(path_master completion zsh)`
    Completion {
        /// The shell for which to generate completions (e.g., bash, zsh, fish, powershell, elvish).
        #[arg(value_hint(ValueHint::CommandString), ignore_case(true), value_names(["SHELL"]))]
        shell: String,
    },
    /// Manages files.
    Files {
        /// Create new files.
        #[arg(long)]
        create: bool,
    },
    /// Manages directories.
    Dirs {
        /// Create new directories.
        #[arg(long)]
        create: bool,
    },
    // If you have a default action when no subcommand is given, you don't need a specific enum variant for it.
    // The `Option<Commands>` handles the case where no subcommand is provided.
}

// The path_helper utility reads the contents of the files in the directories /etc/paths.d and /etc/manpaths.d and appends their contents to the PATH and MANPATH environment variables respectively.  (The MANPATH environment variable will not be modified unless it is already set in the environment.)
fn main() {
    let cli = PathMaster::parse();

    match cli.command {
        Some(Commands::Completion { shell }) => {
            // First, get the `Shell` enum from the string
            let shell_enum = match Shell::from_str(&shell) {
                // Use &shell to borrow
                Ok(s) => s,
                Err(_) => {
                    eprintln!(
                        "Error: Unknown shell '{}'. Supported shells: bash, zsh, fish, powershell, elvish.",
                        shell
                    );
                    std::process::exit(1); // Exit if shell is not recognized
                }
            };

            // Get the top-level `Command` object for your application.
            // PathMaster::command() returns a Command, which needs to be mutable.
            let mut cmd = PathMaster::command();

            // Now call generate with the correct Command object
            generate(
                shell_enum,        // The specific shell generator
                &mut cmd,          // The mutable reference to your top-level Command
                "path_master",     // The binary name
                &mut io::stdout(), // Where to write the completions
            );
        }
        Some(Commands::Files { create }) => {
            if create {
                todo!();
                // handle_files_create();
            } else {
                println!(
                    "'path_master files' called without --create. Add other file operations here."
                );
            }
        }
        Some(Commands::Dirs { create }) => {
            if create {
                todo!();
                // handle_dirs_create();
            } else {
                println!(
                    "'path_master dirs' called without --create. Add other directory operations here."
                );
            }
        }
        None => {
            // This block handles the case where no subcommand is provided
            let dirs = get_paths_dirs_regex(Path::new(ROOTD), PATHSD).unwrap();
            for mut d in dirs {
                d.get_files().unwrap();
                if !d.paths.is_empty() {
                    println!("{}={:?}", d.key, d.path_string().unwrap())
                }
            }
        }
    }
}
