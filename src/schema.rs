use serde::Deserialize;
use tsify::Tsify;

#[derive(Deserialize, Debug, PartialEq, Tsify)]
#[tsify(from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum TableType {
    Gpt,
    Dos,
    None,
}

fn default_sector() -> u64 {
    512
}

#[derive(Deserialize, Debug, Tsify)]
#[tsify(from_wasm_abi)]
#[serde(deny_unknown_fields)]
pub struct SchemaLayout {
    pub table: TableType,
    pub disk_id: Option<String>,
    #[serde(default = "default_sector")]
    pub sector: u64,
    pub partitions: Vec<PartitionConfig>,
}

#[derive(Deserialize, Debug, Tsify)]
#[tsify(from_wasm_abi)]
#[serde(deny_unknown_fields)]
pub struct PartitionConfig {
    #[serde(rename = "type")]
    pub partition_type: Option<String>,
    pub id: String,
    pub offset: Option<u64>,
    pub size: Option<u64>,
    pub hidden: Option<bool>,
}
