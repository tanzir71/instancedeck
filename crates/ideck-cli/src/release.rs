//! Offline verification of InstanceDeck release artifacts only, never app profiles.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use ideck_core::{Error, Result, error::io};
use std::{fs, path::Path};
pub fn verify(file: &Path, version: &str) -> Result<()> {
    let name = file
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::Invalid("Invalid release filename".into()))?;
    if name != "latest.json" && !(name.starts_with("InstanceDeck_") && name.ends_with("-setup.exe"))
    {
        return Err(Error::Invalid(
            "Verification accepts only InstanceDeck setup artifacts or latest.json".into(),
        ));
    }
    let metadata = fs::symlink_metadata(file).map_err(|e| io(file, e))?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > 256 * 1024 * 1024
    {
        return Err(Error::Invalid(
            "Release artifact must be a regular file below 256 MB".into(),
        ));
    }
    let signature_path = file.with_file_name(format!("{name}.sig"));
    let metadata = fs::symlink_metadata(&signature_path).map_err(|e| io(&signature_path, e))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 16 * 1024 {
        return Err(Error::Invalid("Invalid release signature metadata".into()));
    }
    let config: serde_json::Value = serde_json::from_str(include_str!(
        "../../../apps/desktop/src-tauri/tauri.conf.json"
    ))?;
    let key = config["plugins"]["updater"]["pubkey"]
        .as_str()
        .ok_or_else(|| Error::Invalid("Missing project public key".into()))?;
    let key = STANDARD
        .decode(key)
        .map_err(|e| Error::Invalid(e.to_string()))?;
    let key = minisign_verify::PublicKey::decode(
        std::str::from_utf8(&key).map_err(|e| Error::Invalid(e.to_string()))?,
    )
    .map_err(|e| Error::Invalid(e.to_string()))?;
    let signature = STANDARD
        .decode(
            fs::read_to_string(&signature_path)
                .map_err(|e| io(&signature_path, e))?
                .trim(),
        )
        .map_err(|e| Error::Invalid(e.to_string()))?;
    let signature = minisign_verify::Signature::decode(
        std::str::from_utf8(&signature).map_err(|e| Error::Invalid(e.to_string()))?,
    )
    .map_err(|e| Error::Invalid(e.to_string()))?;
    key.verify(&fs::read(file).map_err(|e| io(file, e))?, &signature, false)
        .map_err(|e| Error::Invalid(format!("Integrity signature: {e}")))?;
    let signed = signature
        .trusted_comment()
        .split_whitespace()
        .find_map(|field| field.strip_prefix("version:"))
        .ok_or_else(|| Error::Invalid("Missing signed version".into()))?;
    let parse = |value: &str| {
        semver::Version::parse(value.trim_start_matches('v'))
            .map_err(|e| Error::Invalid(e.to_string()))
    };
    if parse(signed)? != parse(version)? {
        return Err(Error::Invalid(
            "The signed version does not match the announced update version".into(),
        ));
    }
    Ok(())
}
