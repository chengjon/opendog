mod config_output;
mod facade;
mod governance_output;
mod guidance_output;
mod project_output;
mod report_output;
mod verification_output;

pub use self::facade::*;

pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("...{}", &s[s.len() - max + 3..])
    }
}

#[cfg(test)]
mod tests;
