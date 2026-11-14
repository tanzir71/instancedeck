use crate::{Error, Result};

#[derive(Clone, Debug)]
pub struct Context {
    pub profile: String,
    pub install_dir: String,
    pub exe_dir: String,
    pub user_profile: String,
    pub local_app_data: String,
}

pub fn expand(input: &str, context: &Context) -> Result<String> {
    if input.chars().any(|c| matches!(c, '\0' | '\r' | '\n')) {
        return Err(Error::Invalid(
            "Launch templates cannot contain control characters".into(),
        ));
    }
    let mut output = String::new();
    let mut rest = input;
    while let Some(start) = rest.find('{') {
        let prefix = &rest[..start];
        if prefix.contains('}') {
            return Err(Error::Invalid("Unmatched template brace".into()));
        }
        output.push_str(prefix);
        let end = rest[start..]
            .find('}')
            .ok_or_else(|| Error::Invalid("Unclosed template token".into()))?
            + start;
        let value = match &rest[start + 1..end] {
            "profile" => &context.profile,
            "installDir" => &context.install_dir,
            "exeDir" => &context.exe_dir,
            "userProfile" => &context.user_profile,
            "localAppData" => &context.local_app_data,
            _ => return Err(Error::Invalid("Unknown template token".into())),
        };
        output.push_str(value);
        rest = &rest[end + 1..];
    }
    if rest.contains('}') {
        return Err(Error::Invalid("Unmatched template brace".into()));
    }
    output.push_str(rest);
    if output.chars().any(|c| matches!(c, '\0' | '\r' | '\n')) {
        return Err(Error::Invalid(
            "Resolved launch values cannot contain control characters".into(),
        ));
    }
    Ok(output)
}

pub fn validate(input: &str) -> Result<()> {
    let dummy = Context {
        profile: String::new(),
        install_dir: String::new(),
        exe_dir: String::new(),
        user_profile: String::new(),
        local_app_data: String::new(),
    };
    expand(input, &dummy).map(|_| ())
}
