use crate::minecraft_launcher::manifest::main;
use crate::minecraft_launcher::manifest::main::MinVersion;
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Deserialize, Serialize, Clone)]
pub struct Main {
    pub arguments: Option<Arguments>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,
    pub assets: Option<String>,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<u8>,
    pub downloads: Option<Downloads>,
    pub id: String,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    pub libraries: Vec<Library>,
    pub logging: Option<Logging>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: Option<u8>,
    #[serde(rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
    pub time: DateTime<Utc>,
    #[serde(rename = "type")]
    pub _type: VersionType,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
}

impl Main {
    pub fn inherit(mut self, from: &Main) -> Main {
        let from = from.clone();
        match &self.inherits_from {
            None => {}
            Some(vname) => {
                if &from.id != vname {
                    panic!("Trying to inherit from the wrong version!");
                };
            }
        }

        if self.arguments.is_none() {
            if from.arguments.is_some() {
                self.arguments = from.arguments;
            }
        } else if from.arguments.is_some() {
            self.arguments = Some(Arguments::inherit(
                self.arguments.unwrap(),
                from.arguments.unwrap(),
            ));
        }

        if self.asset_index.is_none() && from.asset_index.is_some() {
            self.asset_index = from.asset_index;
        }

        if self.assets.is_none() && from.assets.is_some() {
            self.assets = from.assets;
        }

        if self.compliance_level.is_none() && from.compliance_level.is_some() {
            self.compliance_level = from.compliance_level;
        }

        if self.downloads.is_none() && from.downloads.is_some() {
            self.downloads = from.downloads
        } else if from.downloads.is_some() {
            self.downloads = Some(Downloads::inherit(
                self.downloads.unwrap(),
                from.downloads.unwrap(),
            ));
        }

        if self.java_version.is_none() && from.java_version.is_some() {
            self.java_version = from.java_version;
        }

        for i in from.libraries {
            self.libraries.push(i);
        }

        if self.logging.is_none() && from.logging.is_some() {
            self.logging = from.logging;
        }

        if self.minimum_launcher_version.is_none() && from.minimum_launcher_version.is_some() {
            self.minimum_launcher_version = from.minimum_launcher_version;
        }

        if self.minecraft_arguments.is_none() && from.minecraft_arguments.is_some() {
            self.minecraft_arguments = from.minecraft_arguments;
        }

        self
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Arguments {
    pub game: Vec<Either<String, CustomArguments>>,
    pub jvm: Option<Vec<Either<String, CustomArguments>>>,
}

impl Arguments {
    pub fn inherit(mut self, from: Arguments) -> Arguments {
        for arg in from.game {
            self.game.push(arg);
        }

        if self.jvm.is_none() {
            if from.jvm.is_some() {
                self.jvm = from.jvm;
            }
        } else if from.jvm.is_some() {
            for i in from.jvm.expect("How??") {
                self = self.add_to_jvm(i);
            }
        }

        self
    }

    fn add_to_jvm(mut self, i: Either<String, CustomArguments>) -> Arguments {
        let jvo = self.clone().jvm;
        let mut jv = jvo.expect("");
        jv.push(i);

        self.jvm = Some(jv);

        self
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CustomArguments {
    pub rules: Vec<Rule>,
    pub value: Either<String, Vec<String>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Rule {
    pub action: RuleAction,
    pub features: Option<HashMap<String, bool>>,
    pub os: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub url: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Downloads {
    pub client: DownloadEntry,
    pub client_mappings: Option<DownloadEntry>,
    pub server: Option<DownloadEntry>,
    pub server_mappings: Option<DownloadEntry>,
}

impl Downloads {
    pub fn inherit(mut self, from: Downloads) -> Downloads {
        if self.client_mappings.is_none() && from.client_mappings.is_some() {
            self.client_mappings = from.client_mappings;
        }

        if self.server.is_none() && from.server.is_some() {
            self.server = from.server;
        }

        if self.server_mappings.is_none() && from.server_mappings.is_some() {
            self.server_mappings = from.server_mappings;
        }

        self
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DownloadEntry {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u8,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Library {
    pub downloads: Option<LibraryDownload>,
    pub name: String,
    pub extract: Option<LibraryExtract>,
    pub natives: Option<HashMap<String, String>>,
    pub rules: Option<Vec<Rule>>,
    pub url: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LibraryDownload {
    pub artifact: Option<LibraryDownloadArtifact>,
    pub classifiers: Option<HashMap<String, LibraryDownloadArtifact>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LibraryExtract {
    pub exclude: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LibraryDownloadArtifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Logging {
    pub client: Option<ClientLogging>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ClientLogging {
    pub argument: String,
    pub file: ClientLoggingFile,
    #[serde(rename = "type")]
    pub _type: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ClientLoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub enum VersionType {
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "snapshot")]
    Snapshot,
    #[serde(rename = "old_beta")]
    OldBeta,
    #[serde(rename = "old_alpha")]
    OldAlpha,
    #[serde(rename = "pending")] // Experimental snapshots
    Pending,
}

impl Display for VersionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            VersionType::Release => f.write_str("release"),
            VersionType::Snapshot => f.write_str("snapshot"),
            VersionType::OldBeta => f.write_str("old_beta"),
            VersionType::OldAlpha => f.write_str("old_alpha"),
            VersionType::Pending => f.write_str("pending")
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub enum RuleAction {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "disallow")]
    Disallow,
}

#[derive(Deserialize, Serialize, Clone)]
pub enum Os {
    #[serde(rename = "windows")]
    Windows,
    #[serde(rename = "osx")]
    MacOs,
    #[serde(rename = "linux")]
    Linux,
}

impl Os {
    pub fn from_str(string: &str) -> Option<Os> {
        match string {
            "windows" => Some(Os::Windows),
            "osx" => Some(Os::MacOs),
            "linux" => Some(Os::Linux),
            &_ => None,
        }
    }

    pub fn to_str(&self) -> String {
        String::from(match self {
            Os::Windows => "windows",
            Os::MacOs => "osx",
            Os::Linux => "linux",
        })
    }
}

impl Main {
    pub fn to_min_version(&self) -> MinVersion {
        MinVersion {
            id: self.id.clone(),
            _type: main::VersionType::from_str(self._type.clone().to_string()),
            release_time: self.release_time,
            installed: true,
        }
    }

    pub fn is_modded(&self) -> bool {
        let id = self.id.to_lowercase();
        id.contains("fabric")
            || id.contains("forge")
            || id.contains("liteloader")
            || id.contains("rift")
            || id.contains("optifine")
    }
}

pub fn parse_version_manifest(version_str: &str) -> serde_json::Result<Main> {
    serde_json::from_str(version_str)
}

pub fn serialize_version_manifest(version: &Main) -> serde_json::Result<String> {
    serde_json::to_string_pretty(version)
}
