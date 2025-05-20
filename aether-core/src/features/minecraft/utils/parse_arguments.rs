use daedalus::minecraft;

use crate::features::minecraft::{parse_rules, TEMPORARY_REPLACE_CHAR};

pub fn parse_arguments<F>(
    arguments: &[minecraft::Argument],
    parsed_arguments: &mut Vec<String>,
    parse_function: F,
    java_arch: &str,
) -> crate::Result<()>
where
    F: Fn(&str) -> crate::Result<String>,
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
