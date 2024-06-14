#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct NvH265Config {
    pub name: String,
}

impl Default for NvH265Config {
    fn default() -> Self {
        NvH265Config {
            name: "nvh265enc".to_string(),
        }
    }
}
