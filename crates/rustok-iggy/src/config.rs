use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct IggyConfig {
    #[serde(default)]
    pub mode: IggyMode,
    pub stream: String,
    #[serde(default)]
    pub remote: RemoteConfig,
    #[serde(default)]
    pub embedded: EmbeddedConfig,
    #[serde(default)]
    pub topology: TopologyConfig,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IggyMode {
    #[default]
    Embedded,
    Remote,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RemoteConfig {
    pub api_url: String,
    pub protocol: String,
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            api_url: "127.0.0.1:8090".to_string(),
            protocol: "tcp".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct EmbeddedConfig {
    pub data_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TopologyConfig {
    pub domain_partitions: u32,
    pub replication_factor: u8,
}

impl Default for TopologyConfig {
    fn default() -> Self {
        Self {
            domain_partitions: 4,
            replication_factor: 1,
        }
    }
}
