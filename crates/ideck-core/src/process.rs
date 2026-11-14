use crate::{paths, platform::InstalledApp, recipe::ProcessMatch};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub executable: String,
    pub args: Vec<String>,
    pub started_at: u64,
}

fn matching_value(args: &[String], expected: &str) -> bool {
    let expected = expected.trim_matches('"');
    if let Some((flag, value)) = expected.split_once('=') {
        args.iter().enumerate().any(|(index, arg)| {
            let arg = arg.trim_matches('"');
            if let Some((candidate_flag, candidate_value)) = arg.split_once('=') {
                candidate_flag.eq_ignore_ascii_case(flag)
                    && paths::normalize_windows(candidate_value.trim_matches('"'))
                        == paths::normalize_windows(value)
            } else {
                arg.eq_ignore_ascii_case(flag)
                    && args.get(index + 1).is_some_and(|next| {
                        paths::normalize_windows(next.trim_matches('"'))
                            == paths::normalize_windows(value)
                    })
            }
        })
    } else {
        args.iter().any(|arg| {
            paths::normalize_windows(arg.trim_matches('"')) == paths::normalize_windows(expected)
        })
    }
}

pub fn matches_profile(
    process: &ProcessInfo,
    app: &InstalledApp,
    matcher: &ProcessMatch,
    argument: &str,
) -> bool {
    let name = process
        .executable
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or_default();
    name.eq_ignore_ascii_case(&matcher.exe_name)
        && paths::is_within_windows(&process.executable, &app.install_dir)
        && matching_value(&process.args, argument)
}

pub fn matches_main(process: &ProcessInfo, app: &InstalledApp, matcher: &ProcessMatch) -> bool {
    let name = process
        .executable
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or_default();
    let flag = matcher.arg.split('=').next().unwrap_or_default();
    name.eq_ignore_ascii_case(&matcher.exe_name)
        && paths::is_within_windows(&process.executable, &app.install_dir)
        && !process
            .args
            .iter()
            .any(|arg| arg == "--type" || arg.starts_with("--type="))
        && !process.args.iter().any(|arg| {
            arg.eq_ignore_ascii_case(flag)
                || arg
                    .split('=')
                    .next()
                    .is_some_and(|candidate| candidate.eq_ignore_ascii_case(flag))
        })
}

pub fn tree(processes: &[ProcessInfo], roots: &[u32]) -> Vec<ProcessInfo> {
    let by_pid: BTreeMap<_, _> = processes.iter().map(|p| (p.pid, p)).collect();
    let mut members: BTreeSet<u32> = roots
        .iter()
        .copied()
        .filter(|pid| by_pid.contains_key(pid))
        .collect();
    loop {
        let previous = members.len();
        for process in processes {
            if process.parent_pid.is_some_and(|parent| {
                members.contains(&parent)
                    && by_pid.get(&parent).is_some_and(|parent| {
                        parent.started_at != 0 && process.started_at >= parent.started_at
                    })
            }) {
                members.insert(process.pid);
            }
        }
        if members.len() == previous {
            break;
        }
    }
    processes
        .iter()
        .filter(|p| members.contains(&p.pid))
        .cloned()
        .collect()
}
