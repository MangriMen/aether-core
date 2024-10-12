use daedalus::minecraft;

use crate::launcher::library::parse_rules;

const TEMPORARY_REPLACE_CHAR: &str = "\n";

pub fn parse_arguments<F>(
    arguments: &[minecraft::Argument],
    parsed_arguments: &mut Vec<String>,
    parse_function: F,
    java_arch: &str,
) -> anyhow::Result<()>
where
    F: Fn(&str) -> anyhow::Result<String>,
{
    for argument in arguments {
        match argument {
            minecraft::Argument::Normal(arg) => {
                let parsed = parse_function(&arg.replace(' ', TEMPORARY_REPLACE_CHAR))?;
                for arg in parsed.split(TEMPORARY_REPLACE_CHAR) {
                    parsed_arguments.push(arg.to_string());
                }
            }
            minecraft::Argument::Ruled { rules, value } => {
                if parse_rules(rules, java_arch, true) {
                    match value {
                        minecraft::ArgumentValue::Single(arg) => {
                            parsed_arguments
                                .push(parse_function(&arg.replace(' ', TEMPORARY_REPLACE_CHAR))?);
                        }
                        minecraft::ArgumentValue::Many(args) => {
                            for arg in args {
                                parsed_arguments.push(parse_function(
                                    &arg.replace(' ', TEMPORARY_REPLACE_CHAR),
                                )?);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
