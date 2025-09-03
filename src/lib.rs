//! Autogenerate Markdown documentation for clap command-line tools
//!
//! See [**Examples**][Examples] for examples of the content `clap-markdown`
//! generates.
//!
//! [Examples]: https://github.com/ConnorGray/clap-markdown#Examples
//!

// Ensure that doc tests in the README.md file get run.

mod utils;
use clap::builder::PossibleValue;
use std::io::Write;
use utils::pluralize;

/// Generate mdx page files for the command with all subcommands
pub fn generate_to(
    command: &clap::Command,
    out_dir: impl AsRef<std::path::Path>,
) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    fn generate(
        command: &clap::Command,
        parent_command_path: &Vec<String>,
        out_dir: &std::path::Path,
    ) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
        let command_name = get_canonical_name(command);
        std::fs::create_dir_all(&out_dir)?;
        let mut paths = Vec::new();
        paths.push(generate_command_to(
            command,
            &parent_command_path,
            if parent_command_path.is_empty() {
                out_dir.join("index.mdx")
            } else {
                out_dir.join(format!("{}.mdx", command_name))
            },
        )?);

        let out_dir = out_dir.join(&command_name);
        let command_path = {
            let mut command_path = parent_command_path.clone();
            command_path.push(command_name);
            command_path
        };

        for subcommand in command.get_subcommands().filter(|s| !s.is_hide_set())
        {
            paths.extend(generate(subcommand, &command_path, &out_dir)?);
        }
        Ok(paths)
    }

    generate(command, &Vec::new(), out_dir.as_ref())
}

pub fn generate_command_to(
    command: &clap::Command,
    parent_command_path: &Vec<String>,
    path: impl AsRef<std::path::Path>,
) -> Result<std::path::PathBuf, std::io::Error> {
    let mut file = std::fs::File::create(&path)?;
    write_command_markdown(&mut file, parent_command_path, command)?;
    file.flush()?;
    Ok(path.as_ref().into())
}

pub fn write_command_markdown(
    w: &mut dyn Write,
    parent_command_path: &Vec<String>,
    command: &clap::Command,
) -> Result<(), std::io::Error> {
    let title_name = get_canonical_name(command);

    // Append the name of `command` to `command_path`.
    let command_path = {
        let mut command_path = parent_command_path.clone();
        command_path.push(title_name);
        command_path
    };
    let command_path_str = command_path.join("/");

    //----------------------------------
    // Write the markdown heading
    //----------------------------------

    writeln!(w, "---")?;
    writeln!(w, "title: {}", command_path.join(" "))?;
    writeln!(w, "isCommand: true")?;
    writeln!(w, "---\n")?;

    if let Some(long_about) = command.get_long_about() {
        writeln!(w, "{}\n", long_about)?;
    } else if let Some(about) = command.get_about() {
        writeln!(w, "{}\n", about)?;
    }

    if let Some(help) = command.get_before_long_help() {
        writeln!(w, "{}\n", help)?;
    } else if let Some(help) = command.get_before_help() {
        writeln!(w, "{}\n", help)?;
    }

    writeln!(
        w,
        "```shell title=\"Usage\"\n{}{}\n```\n",
        if parent_command_path.is_empty() {
            String::new()
        } else {
            let mut s = parent_command_path.join(" ");
            s.push_str(" ");
            s
        },
        command
            .clone()
            .render_usage()
            .to_string()
            .replace("Usage: ", "")
    )?;

    let aliases = command.get_visible_aliases().collect::<Vec<&str>>();
    if let Some(aliases_str) = get_alias_string(&aliases) {
        writeln!(
            w,
            "**{}:** {aliases_str}\n",
            pluralize(aliases.len(), "Command Alias", "Command Aliases")
        )?;
    }

    if let Some(help) = command.get_after_long_help() {
        writeln!(w, "{}\n", help)?;
    } else if let Some(help) = command.get_after_help() {
        writeln!(w, "{}\n", help)?;
    }

    //----------------------------------
    // Subcommands
    //----------------------------------

    if command.get_subcommands().next().is_some() {
        writeln!(w, "### Subcommands\n")?;

        for subcommand in command.get_subcommands() {
            if subcommand.is_hide_set() {
                continue;
            }

            let title_name = get_canonical_name(subcommand);

            let about = match subcommand.get_about() {
                Some(about) => about.to_string(),
                None => String::new(),
            };

            writeln!(
                w,
                "- [`{title_name}`](./{command_path_str}/{title_name}) — {about}",
            )?;
        }

        write!(w, "\n")?;
    }

    //----------------------------------
    // Arguments
    //----------------------------------

    if command.get_positionals().next().is_some() {
        writeln!(w, "### Arguments\n")?;

        for pos_arg in command.get_positionals() {
            write_arg_markdown(w, pos_arg)?;
        }

        write!(w, "\n")?;
    }

    //----------------------------------
    // Options
    //----------------------------------

    let non_pos: Vec<_> = command
        .get_arguments()
        .filter(|arg| !arg.is_positional() && !arg.is_hide_set())
        .collect();

    if !non_pos.is_empty() {
        writeln!(w, "### Options\n")?;

        for arg in non_pos {
            write_arg_markdown(w, arg)?;
        }

        write!(w, "\n")?;
    }

    Ok(())
}

fn write_arg_markdown(
    w: &mut dyn Write,
    arg: &clap::Arg,
) -> Result<(), std::io::Error> {
    // Markdown list item
    write!(w, "- ")?;

    let value_name: String = match arg.get_value_names() {
        // TODO: What if multiple names are provided?
        Some([name, ..]) => name.as_str().to_owned(),
        Some([]) => unreachable!(
            "clap Arg::get_value_names() returned Some(..) of empty list"
        ),
        None => arg.get_id().to_string().to_ascii_uppercase(),
    };

    match (arg.get_short(), arg.get_long()) {
        (Some(short), Some(long)) => {
            if arg.get_action().takes_values() {
                write!(w, "`-{short}`, `--{long} <{value_name}>`")?
            } else {
                write!(w, "`-{short}`, `--{long}`")?
            }
        },
        (Some(short), None) => {
            if arg.get_action().takes_values() {
                write!(w, "`-{short} <{value_name}>`")?
            } else {
                write!(w, "`-{short}`")?
            }
        },
        (None, Some(long)) => {
            if arg.get_action().takes_values() {
                write!(w, "`--{} <{value_name}>`", long)?
            } else {
                write!(w, "`--{}`", long)?
            }
        },
        (None, None) => {
            debug_assert!(arg.is_positional(), "unexpected non-positional Arg with neither short nor long name: {arg:?}");

            write!(w, "`<{value_name}>`",)?;
        },
    }

    if let Some(aliases) = arg.get_visible_aliases().as_deref() {
        if let Some(aliases_str) = get_alias_string(aliases) {
            write!(
                w,
                " [{}: {aliases_str}]",
                pluralize(aliases.len(), "alias", "aliases")
            )?;
        }
    }

    if let Some(help) = arg.get_long_help() {
        // TODO: Parse formatting in the string
        write!(w, "{}", &indent(&help.to_string(), " — ", "  "))?;
    } else if let Some(short_help) = arg.get_help() {
        writeln!(w, " — {short_help}")?;
    } else {
        writeln!(w)?;
    }

    //--------------------
    // Arg default values
    //--------------------

    if !arg.get_default_values().is_empty() {
        let default_values: String = arg
            .get_default_values()
            .iter()
            .map(|value| format!("`{}`", value.to_string_lossy()))
            .collect::<Vec<String>>()
            .join(", ");

        if arg.get_default_values().len() > 1 {
            // Plural
            writeln!(w, "\n  Default values: {default_values}")?;
        } else {
            // Singular
            writeln!(w, "\n  Default value: {default_values}")?;
        }
    }

    //--------------------
    // Arg possible values
    //--------------------

    let possible_values: Vec<PossibleValue> = arg
        .get_possible_values()
        .into_iter()
        .filter(|pv| !pv.is_hide_set())
        .collect();

    // Print possible values for options that take a value, but not for flags
    // that can only be either present or absent and do not take a value.
    if !possible_values.is_empty()
        && !matches!(arg.get_action(), clap::ArgAction::SetTrue)
    {
        let any_have_help: bool =
            possible_values.iter().any(|pv| pv.get_help().is_some());

        if any_have_help {
            // If any of the possible values have help text, print them
            // as a separate item in a bulleted list, and include the
            // help text for those that have it. E.g.:
            //
            //     Possible values:
            //     - `value1`:
            //       The help text
            //     - `value2`
            //     - `value3`:
            //       The help text

            let text: String = possible_values
                .iter()
                .map(|pv| match pv.get_help() {
                    Some(help) => {
                        format!("  - `{}`:\n    {}\n", pv.get_name(), help)
                    },
                    None => format!("  - `{}`\n", pv.get_name()),
                })
                .collect::<Vec<String>>()
                .join("");

            writeln!(w, "\n  Possible values:\n\n{text}")?;
        } else {
            // If none of the possible values have any documentation, print
            // them all inline on a single line.
            let text: String = possible_values
                .iter()
                // TODO: Show PossibleValue::get_help(), and PossibleValue::get_name_and_aliases().
                .map(|pv| format!("`{}`", pv.get_name()))
                .collect::<Vec<String>>()
                .join(", ");

            writeln!(w, "\n  Possible values: {text}\n")?;
        }
    }

    Ok(())
}

/// Utility function to get the canonical name of a command.
///
/// It's logic is to get the display name if it exists, otherwise get the bin
/// name if it exists, otherwise get the package name.
///
/// Note that the default `Command.name` field of a clap command is typically
/// meant for programmatic usage as well as for display (if no `display_name`
/// override is set).
fn get_canonical_name(command: &clap::Command) -> String {
    command
        .get_display_name()
        .or_else(|| command.get_bin_name())
        .map(|name| name.to_owned())
        .unwrap_or_else(|| command.get_name().to_owned())
}

/// Indents non-empty lines. The output always ends with a newline.
fn indent(s: &str, first: &str, rest: &str) -> String {
    if s.is_empty() {
        // For consistency. It's easiest to always add a newline at the end, and
        // there's little reason not to.
        return "\n".to_string();
    }
    let mut result = String::new();
    let mut first_line = true;

    for line in s.lines() {
        if !line.is_empty() {
            result.push_str(if first_line { first } else { rest });
            result.push_str(line);
            first_line = false;
        }
        result.push('\n');
    }
    result
}

fn get_alias_string(aliases: &[&str]) -> Option<String> {
    if aliases.is_empty() {
        return None;
    }

    Some(format!(
        "{}",
        aliases
            .iter()
            .map(|alias| format!("`{alias}`"))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

#[cfg(test)]
mod test {

    #[test]
    fn test_indent() {
        use super::indent;
        assert_eq!(
            &indent("Header\n\nMore info", "___", "~~~~"),
            "___Header\n\n~~~~More info\n"
        );
        assert_eq!(
            &indent("Header\n\nMore info\n", "___", "~~~~"),
            &indent("Header\n\nMore info", "___", "~~~~"),
        );
        assert_eq!(&indent("", "___", "~~~~"), "\n");
        assert_eq!(&indent("\n", "___", "~~~~"), "\n");
    }
}
