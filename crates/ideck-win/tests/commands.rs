use ideck_core::{InstalledApp, LaunchPlan};
use ideck_win::command::*;
use std::collections::BTreeMap;

fn plan() -> LaunchPlan {
    LaunchPlan {
        profile_id: "fixture".into(),
        recipe_id: "fixture".into(),
        recipe_version: 1,
        app: InstalledApp {
            app_id: "fixture".into(),
            name: "Fixture".into(),
            version: "1".into(),
            executable: r"C:\Program Files\Fixture\app.exe".into(),
            install_dir: r"C:\Program Files\Fixture".into(),
            install_type: "msix".into(),
            package_family: Some("Fixture_Family".into()),
            package_app_id: Some("App".into()),
        },
        args: vec![
            r"--data=D:\Zoë 日本語\profile with spaces".into(),
            "a 'quote' & %PATH% ! literal".into(),
        ],
        env: BTreeMap::from([(
            "FIXTURE_DATA".into(),
            r"D:\Zoë 日本語\it's a profile & %not-an-expansion% !".into(),
        )]),
        working_dir: r"C:\Program Files\Fixture".into(),
        directories: vec![],
        match_argument: String::new(),
    }
}

#[test]
fn windows_quoting_doubles_only_backslashes_before_quotes_and_the_end() {
    assert_eq!(quote_windows(""), "\"\"");
    assert_eq!(quote_windows("space path"), "\"space path\"");
    assert_eq!(quote_windows("say \"yes\""), "\"say \\\"yes\\\"\"");
    assert_eq!(quote_windows("C:\\ending\\"), "\"C:\\ending\\\\\"");
    assert_eq!(ps_literal("it's 'literal'"), "'it''s ''literal'''");
}

#[test]
fn wrappers_are_utf16_bom_and_encode_every_untrusted_value() {
    let plan = plan();
    let script = launch_script(&plan).unwrap();
    assert!(script.contains("it''s a profile"));
    assert!(script.contains("UseShellExecute = $false"));
    assert!(script.contains("CreateNoWindow = $true"));
    let line = wrapper_line(
        r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
        &plan,
    )
    .unwrap();
    assert!(!line.contains("Zoë"));
    assert!(!line.contains("%PATH%"));
    assert!(!line.contains('&'));
    let bytes = utf16_le(&line, true);
    assert_eq!(&bytes[..2], &[0xff, 0xfe]);
    let restored = String::from_utf16(
        &bytes[2..]
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    assert_eq!(restored, line);
    let package = package_script(
        "Fixture_Family",
        "App",
        "powershell.exe",
        r"C:\Users\Zoë Example\run\it's a wrapper.cmd",
    );
    assert!(package.contains("-PreventBreakaway"));
    assert!(!package.contains("Zoë"));
    let mut invalid = plan;
    invalid.args.push("line\nbreak".into());
    assert!(launch_script(&invalid).is_err());
}

#[cfg(windows)]
#[test]
fn unicode_wrapper_launches_a_fixture_with_literal_environment_values() {
    use std::{fs, os::windows::process::CommandExt, process::Command, time::Duration};
    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("Zoë 日本語 with spaces & apostrophe's");
    fs::create_dir_all(&directory).unwrap();
    let output = directory.join("fixture.json");
    let powershell = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap())
        .join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let fixture = format!(
        "[IO.File]::WriteAllText({}, $env:FIXTURE_DATA, [Text.Encoding]::UTF8)",
        ps_literal(&output.to_string_lossy())
    );
    let mut plan = plan();
    plan.app.executable = powershell.to_string_lossy().into_owned();
    plan.working_dir = directory.to_string_lossy().into_owned();
    plan.args = vec![
        "-NoProfile".into(),
        "-NonInteractive".into(),
        "-EncodedCommand".into(),
        encoded_powershell(&fixture),
    ];
    let wrapper = directory.join("launch fixture.cmd");
    fs::write(
        &wrapper,
        utf16_le(
            &wrapper_line(&powershell.to_string_lossy(), &plan).unwrap(),
            true,
        ),
    )
    .unwrap();
    // This is the same Unicode loader used inside the package-context host.
    let loader = format!(
        "$ErrorActionPreference='Stop'; $line=[IO.File]::ReadAllText({}, [Text.Encoding]::Unicode).Trim(); & $env:ComSpec /D /S /V:OFF /C ('\"' + $line + '\"'); exit $LASTEXITCODE",
        ps_literal(&wrapper.to_string_lossy())
    );
    let status = Command::new(&powershell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-EncodedCommand",
            &encoded_powershell(&loader),
        ])
        .creation_flags(0x08000000)
        .status()
        .unwrap();
    assert!(status.success());
    for _ in 0..60 {
        if output.exists() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let actual = fs::read_to_string(&output).unwrap();
    assert_eq!(
        actual.trim_start_matches('\u{feff}'),
        plan.env["FIXTURE_DATA"]
    );
}
