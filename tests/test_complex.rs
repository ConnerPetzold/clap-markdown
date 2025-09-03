use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};
use clap_markdown::{generate_command_to, generate_to};
use insta::assert_debug_snapshot;

/// An example command-line tool
#[derive(Parser)]
#[command(name = "complex-app", visible_aliases = ["ca"])]
pub struct Cli {
    /// Optional name to operate on
    ///
    /// Longer description
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE", visible_alias = "configuration")]
    config: Option<PathBuf>,

    #[arg(long, default_value = "local")]
    target: Target,

    #[arg(long, visible_alias = "vv", visible_alias = "vvv")]
    very_very_verbose: bool,

    /// Turn debugging information on
    ///
    /// Repeat this option to see more and more debug information.
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[arg(short, long, hide = true)]
    secret_arg: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    #[command(visible_alias = "tester")]
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
    /// Demo that `Options` is not printed if all options are hidden
    OnlyHiddenOptions {
        #[arg(short, long, hide = true)]
        secret: bool,
    },
}

#[derive(clap::ValueEnum)]
#[derive(Clone)]
enum Target {
    /// Do the operation locally
    Local,
    // Intentionally undocumented.
    Remote,
}

#[test]
fn test_example_complex_app() {
    assert_debug_snapshot!(
        generate_to(
            &Cli::command(),
            PathBuf::from("tests/complex-app"),
        ).unwrap(),
        @r#"
    [
        "tests/complex-app/index.mdx",
        "tests/complex-app/complex-app/test.mdx",
        "tests/complex-app/complex-app/only-hidden-options.mdx",
    ]
    "#
    );
}
