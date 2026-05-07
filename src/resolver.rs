use crate::layout::ResolvedPartition;
use crate::schema::{SchemaLayout, TableType};
use std::collections::{BTreeMap, HashSet};

pub struct ResolvedInfo {
    pub disk_size: u64,
    pub partitions: Vec<ResolvedPartition>,
}

pub fn resolve(
    schema: &SchemaLayout,
    file_sizes: &BTreeMap<String, u64>,
) -> Result<ResolvedInfo, String> {
    let sector_size = schema.sector;

    if let Some(did) = &schema.disk_id {
        match schema.table {
            TableType::Gpt => crate::utils::parse_uuid_to_gpt_guid(did)
                .map_err(|e| format!("disk_id must be a valid UUID: {}", e))?,
            TableType::Dos => {
                if u32::from_str_radix(did.trim_start_matches("0x"), 16).is_err() {
                    return Err(
                        "disk_id must be a valid 32-bit Hex number (e.g. 0x12345678)".into(),
                    );
                }
                [0; 16] // Dummy
            }
            TableType::None => [0; 16], // Dummy
        };
    }

    let mut ids = HashSet::new();
    let mut current_lba = 2048;
    let mut last_end: Option<u64> = None;
    let mut visible_count = 0;

    let mut min_required_bytes: u64 = 0;
    let mut resolved_partitions = Vec::new();

    for item in &schema.partitions {
        if !ids.insert(&item.id) {
            return Err(format!("Duplicate partition ID: {}", item.id));
        }

        if !item.hidden.unwrap_or(false) {
            match schema.table {
                TableType::Gpt => {
                    let ptype = item
                        .partition_type
                        .as_ref()
                        .ok_or_else(|| format!("GPT partition {} missing type (UUID)", item.id))?;
                    crate::utils::parse_uuid_to_gpt_guid(ptype).map_err(|e| {
                        format!("Partition {} type is not a valid UUID: {}", item.id, e)
                    })?;
                }
                TableType::Dos => {
                    let ptype = item
                        .partition_type
                        .as_ref()
                        .ok_or_else(|| format!("DOS partition {} missing type (Hex)", item.id))?;
                    if u8::from_str_radix(ptype.trim_start_matches("0x"), 16).is_err() {
                        return Err(format!(
                            "Partition {} type is not a valid DOS hex format",
                            item.id
                        ));
                    }
                }
                TableType::None => {}
            }
            visible_count += 1;
        }

        if let Some(off) = item.offset {
            if off % sector_size != 0 {
                return Err(format!(
                    "Partition {} offset is not aligned to sector",
                    item.id
                ));
            }
        }

        let file_size = file_sizes.get(&item.id).copied().unwrap_or(0);

        let actual_size = match item.size {
            Some(s) => {
                let aligned = s
                    .div_ceil(sector_size)
                    .checked_mul(sector_size)
                    .ok_or_else(|| format!("Partition {} size calculation overflow", item.id))?;

                if file_size > aligned {
                    return Err(format!(
                        "Partition {} input file exceeds preset size",
                        item.id
                    ));
                }
                aligned
            }
            None => {
                if file_size == 0 {
                    return Err(format!(
                        "Partition {} missing size and no input file to infer",
                        item.id
                    ));
                }
                file_size
                    .div_ceil(sector_size)
                    .checked_mul(sector_size)
                    .ok_or_else(|| format!("Partition {} size calculation overflow", item.id))?
            }
        };

        let start_lba = if let Some(off) = item.offset {
            off / sector_size
        } else {
            if current_lba % 2048 != 0 {
                current_lba = ((current_lba / 2048) + 1) * 2048;
            }
            current_lba
        };

        if let Some(le) = last_end {
            if start_lba <= le {
                return Err(format!(
                    "Partition {} position overlaps or is out of order with previous partition",
                    item.id
                ));
            }
        }

        let lba_size = actual_size / sector_size;
        let end_lba = start_lba + lba_size - 1;

        if schema.table == TableType::Dos && end_lba > u32::MAX as u64 {
            return Err(format!(
                "Partition {} exceeds DOS MBR capacity limit",
                item.id
            ));
        }

        last_end = Some(end_lba);
        current_lba = end_lba + 1;

        // 记录推导完毕的分区数据
        let offset = start_lba * sector_size;
        resolved_partitions.push(ResolvedPartition {
            id: item.id.clone(),
            offset,
            size: actual_size,
        });

        let end_bytes = offset
            .checked_add(actual_size)
            .ok_or_else(|| format!("Partition {} space range exceeds u64 limit", item.id))?;
        if end_bytes > min_required_bytes {
            min_required_bytes = end_bytes;
        }
    }

    if schema.table == TableType::Gpt && visible_count > 128 {
        return Err("GPT format supports at most 128 partition entries".to_string());
    }
    if schema.table == TableType::Dos && visible_count > 4 {
        return Err("DOS MBR supports at most 4 primary partition entries".to_string());
    }

    min_required_bytes += 33 * sector_size;
    let auto_disk_size = min_required_bytes.div_ceil(1048576) * 1048576;

    Ok(ResolvedInfo {
        disk_size: auto_disk_size,
        partitions: resolved_partitions,
    })
}
