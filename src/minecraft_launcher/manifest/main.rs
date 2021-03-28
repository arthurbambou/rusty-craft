use serde;
use serde_derive::Deserialize;

use chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};

#[derive(Deserialize)]
pub struct Main {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(Deserialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize)]
pub struct Version {
    pub id: String,
    #[serde(alias = "type")]
    pub _type: VersionType,
    pub url: String,
    pub time: DateTime<Utc>,
    #[serde(alias = "releaseTime")]
    pub release_time: DateTime<Utc>,
    pub sha1: String,
    #[serde(alias = "complianceLevel")]
    pub compliance_level: u8,
}

#[derive(Clone)]
pub struct MinVersion {
    pub id: String,
    pub _type: VersionType,
    pub release_time: DateTime<Utc>
}

#[derive(Deserialize, Clone)]
pub enum VersionType {
    #[serde(alias="release")]
    Release,
    #[serde(alias="snapshot")]
    Snapshot,
    #[serde(alias="old_beta")]
    OldBeta,
    #[serde(alias="old_alpha")]
    OldAlpha
}

impl Display for VersionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            VersionType::Release => {f.write_str("release")}
            VersionType::Snapshot => {f.write_str("snapshot")}
            VersionType::OldBeta => {f.write_str("old_beta")}
            VersionType::OldAlpha => {f.write_str("old_alpha")}
        }
    }
}

impl VersionType {
    pub fn from_str(string: String) -> VersionType {
        match string.as_str() {
            "release" => VersionType::Release,
            "snapshot" => VersionType::Snapshot,
            "old_beta" => VersionType::OldBeta,
            "old_alpha" => VersionType::OldAlpha,
            _ => {panic!("Unknown version type: {}", string)}
        }
    }
}

impl Version {
    pub fn to_min_version(&self) -> MinVersion {
        MinVersion {
            id: self.id.clone(),
            _type: self._type.clone(),
            release_time: self.release_time
        }
    }
}

pub fn parse_manifest(manifest_str: &String) -> serde_json::Result<Main> {
    let manifest: serde_json::Result<Main> = serde_json::from_str(manifest_str);

    manifest
}
