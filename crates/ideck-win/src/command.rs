//! Pure command construction. All user-supplied values remain quoted data.
use base64::Engine as _;
use ideck_core::{Error, LaunchPlan, Result};

pub fn quote_windows(value: &str) -> String {
    let mut output = String::from("\"");
    let mut slashes = 0;
    for character in value.chars() {
        match character {
            '\\' => slashes += 1,
            '"' => {
                output.push_str(&"\\".repeat(slashes * 2 + 1));
                output.push('"');
                slashes = 0;
            }
            _ => {
                output.push_str(&"\\".repeat(slashes));
                output.push(character);
                slashes = 0;
            }
        }
    }
    output.push_str(&"\\".repeat(slashes * 2));
    output.push('"');
    output
}

pub fn ps_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub fn utf16_le(input: &str, bom: bool) -> Vec<u8> {
    let mut bytes = if bom { vec![0xff, 0xfe] } else { Vec::new() };
    bytes.extend(input.encode_utf16().flat_map(u16::to_le_bytes));
    bytes
}

pub fn encoded_powershell(script: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(utf16_le(script, false))
}

pub fn launch_script(plan: &LaunchPlan) -> Result<String> {
    if plan
        .args
        .iter()
        .chain(plan.env.values())
        .chain([&plan.app.executable, &plan.working_dir])
        .any(|value| value.chars().any(|c| matches!(c, '\0' | '\r' | '\n')))
    {
        return Err(Error::Invalid(
            "Package launch values cannot contain control characters".into(),
        ));
    }
    let mut script = String::from("$ErrorActionPreference = 'Stop';\n");
    for (name, value) in &plan.env {
        script.push_str(&format!(
            "[Environment]::SetEnvironmentVariable({}, {}, 'Process');\n",
            ps_literal(name),
            ps_literal(value)
        ));
    }
    let arguments = plan
        .args
        .iter()
        .map(|arg| quote_windows(arg))
        .collect::<Vec<_>>()
        .join(" ");
    script.push_str(&format!(
        "$start = New-Object System.Diagnostics.ProcessStartInfo; $start.FileName = {}; $start.WorkingDirectory = {}; $start.Arguments = {}; $start.UseShellExecute = $false; $start.CreateNoWindow = $true; [Diagnostics.Process]::Start($start) | Out-Null;\n",
        ps_literal(&plan.app.executable),
        ps_literal(&plan.working_dir),
        ps_literal(&arguments)
    ));
    Ok(script)
}

/// UTF-16 wrapper is decoded before passing its ASCII-only launch line to cmd.
/// cmd.exe cannot reliably interpret a UTF-16 batch file directly.
pub fn wrapper_line(powershell: &str, plan: &LaunchPlan) -> Result<String> {
    Ok(format!(
        "{} -NoProfile -NonInteractive -EncodedCommand {}",
        quote_windows(powershell),
        encoded_powershell(&launch_script(plan)?)
    ))
}

pub fn package_script(family: &str, app_id: &str, powershell: &str, wrapper_path: &str) -> String {
    let loader = format!(
        "$ErrorActionPreference='Stop'; $line=[IO.File]::ReadAllText({}, [Text.Encoding]::Unicode).Trim(); & $env:ComSpec /D /S /V:OFF /C ('\"' + $line + '\"'); if ($LASTEXITCODE -ne 0) {{ exit $LASTEXITCODE }}",
        ps_literal(wrapper_path)
    );
    let arguments = format!(
        "-NoProfile -NonInteractive -EncodedCommand {}",
        encoded_powershell(&loader)
    );
    format!(
        "$ErrorActionPreference='Stop'; Invoke-CommandInDesktopPackage -PackageFamilyName {} -AppId {} -Command {} -Args {} -PreventBreakaway;",
        ps_literal(family),
        ps_literal(app_id),
        ps_literal(powershell),
        ps_literal(&arguments)
    )
}
