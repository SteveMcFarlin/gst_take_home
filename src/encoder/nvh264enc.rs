#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct NvH264Config {
    pub name: String,
}

impl Default for NvH264Config {
    fn default() -> Self {
        NvH264Config {
            name: "nvh265enc".to_string(),
        }
    }
}
