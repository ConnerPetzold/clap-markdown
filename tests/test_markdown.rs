use std::path::PathBuf;

use clap::{Arg, Command};
use clap_markdown::generate_to;

use insta::{assert_debug_snapshot, assert_snapshot};

/// Test behavior for an app that defines a:
///
/// * `name`
/// * `display_name`
///
/// but no custom `bin_name`.
#[test]
fn test_title_behavior_for_name_and_display_name_app() {
    let mut app = Command::new("my-program-name")
        // Note: Intentionally not setting custom bin name.
        // .bin_name("my-program-bin-name")
        .display_name("my-program-display-name")
        .version("1.2.3")
        .about("This program does things.")
        .arg(Arg::new("foo").short('f'));
    let () = app.build();

    //-------------------------------------------------------------------
    // Test the native behavior of `clap`, in terms of whether it prefers
    // the `name`, `bin_name`, and `display_name` properties are used.
    //-------------------------------------------------------------------

    assert_snapshot!(
        app.render_long_help().to_string(),
        @r"
    This program does things.

    Usage: my-program-name [OPTIONS]

    Options:
      -f <foo>
              

      -h, --help
              Print help

      -V, --version
              Print version
    ");

    //-------------------------------------------------------
    // Test how clap-markdown handles the various name fields
    //-------------------------------------------------------

    assert_debug_snapshot!(
        generate_to(
            &app,
            PathBuf::from("tests"),
        ).unwrap(),
        @r#"
    [
        "tests/my-program-display-name.mdx",
    ]
    "#);
}
