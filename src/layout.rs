use serde::Serialize;
use tsify::Tsify;

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct ResolvedPartition {
    pub id: String,
    pub offset: u64,
    pub size: u64,
}

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct LayoutResult {
    pub disk_size: u64, // to frontend
    pub metadata_chunks: Vec<Chunk>,
    pub resolved_partitions: Vec<ResolvedPartition>,
}

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct Chunk {
    pub offset: u64,
    pub data: Vec<u8>,
}
