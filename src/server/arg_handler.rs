use std::sync::Arc;
use clap::Parser;

/// `ArgsCli` is an alias for an `Arc`-wrapped `ArgHandler`, which holds the command-line arguments.
pub type ArgsCli = Arc<ArgHandler>;

/// `ArgHandler` is a struct that represents the command-line arguments for the application.
///
/// The struct uses `clap` for parsing command-line arguments and derives from `Parser`,
/// which provides the necessary functionality to handle command-line input.
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct ArgHandler {
    /// Directory path provided by the user as a command-line argument.
    #[arg(long)]
    pub dir: Option<String>,

    /// Database filename provided by the user as a command-line argument.
    #[arg(long)]
    pub dbfilename: Option<String>,
}

impl ArgHandler {
    /// Parses the command-line arguments and returns them wrapped in an `Arc`.
    ///
    /// # Returns
    ///
    /// Returns an `ArgsCli` type, which is an `Arc` containing the parsed `ArgHandler`.
    ///
    /// # Examples
    ///
    /// ```
    /// let args = ArgHandler::retrieve_args();
    /// ```
    pub fn retrieve_args() -> ArgsCli {
        Arc::new(ArgHandler::parse())
    }

    /// Checks if both `dir` and `dbfilename` are provided by the user.
    ///
    /// # Returns
    ///
    /// Returns `true` if both `dir` and `dbfilename` are `Some`, otherwise returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// let args = ArgHandler::retrieve_args();
    /// if args.can_be_parsed() {
    ///     // Proceed with further logic
    /// }
    /// ```
    pub fn can_be_parsed(&self) -> bool {
        self.dir.is_some() && self.dbfilename.is_some()
    }
}
