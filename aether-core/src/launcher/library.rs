use daedalus::minecraft;
use regex::Regex;

pub fn parse_rules(rules: &[minecraft::Rule], java_version: &str, minecraft_updated: bool) -> bool {
    let mut parse_results = rules
        .iter()
        .map(|rule| parse_rule(rule, java_version, minecraft_updated))
        .collect::<Vec<Option<bool>>>();

    if rules
        .iter()
        .all(|rule| matches!(rule.action, minecraft::RuleAction::Disallow))
    {
        parse_results.push(Some(true))
    }

    !(parse_results.iter().any(|x| x == &Some(false)) || parse_results.iter().all(|x| x.is_none()))
}

fn parse_rule(rule: &minecraft::Rule, java_version: &str, minecraft_updated: bool) -> Option<bool> {
    use minecraft::{Rule, RuleAction};

    let res = match rule {
        Rule {
            os: Some(ref os), ..
        } => parse_os_rule(os, java_version, minecraft_updated),
        Rule {
            features: Some(ref features),
            ..
        } => {
            !features.is_demo_user.unwrap_or(true)
                || features.has_custom_resolution.unwrap_or(false)
                || !features.has_quick_plays_support.unwrap_or(true)
                || !features.is_quick_play_multiplayer.unwrap_or(true)
                || !features.is_quick_play_realms.unwrap_or(true)
                || !features.is_quick_play_singleplayer.unwrap_or(true)
        }
        _ => return Some(true),
    };

    match rule.action {
        RuleAction::Allow => Some(res),
        RuleAction::Disallow if res => Some(false),
        RuleAction::Disallow => None,
    }
}

// Platform rule resolving
pub fn parse_os_rule(
    rule: &minecraft::OsRule,
    java_arch: &str,
    // Minecraft updated over 1.18.2 (supports MacOS Natively)
    minecraft_updated: bool,
) -> bool {
    use crate::utils::platform::OsExt;
    use minecraft::Os;

    let mut rule_match = true;

    // Check architecture
    if let Some(ref arch) = rule.arch {
        rule_match &= !matches!(arch.as_str(), "x86" | "arm");
    }

    // Check OS name
    if let Some(name) = &rule.name {
        // If Minecraft is updated and the OS name is not "LinuxArm64" or "LinuxArm32",
        // the rule matches if the native OS or the OS corresponding to the Java architecture matches the OS name
        if minecraft_updated && (name != &Os::LinuxArm64 || name != &Os::LinuxArm32) {
            rule_match &= &Os::native() == name || &Os::native_arch(java_arch) == name;
        } else {
            // If Minecraft is not updated, the rule matches if the OS corresponding to the Java architecture matches the OS name
            rule_match &= &Os::native_arch(java_arch) == name;
        }
    }

    // Check OS version
    if let Some(version) = &rule.version {
        if let Ok(regex) = Regex::new(version.as_str()) {
            rule_match &= regex.is_match(&sys_info::os_release().unwrap_or_default());
        }
    }

    rule_match
}

#[macro_export]
macro_rules! processor_rules {
    ($dest:expr; $($name:literal : client => $client:expr, server => $server:expr;)+) => {
        $(std::collections::HashMap::insert(
            $dest,
            String::from($name),
            daedalus::modded::SidedDataEntry {
                client: String::from($client),
                server: String::from($server),
            },
        );)+
    }
}

pub(crate) use processor_rules;
