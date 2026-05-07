use crate::disk::{SparseDisk, BLOCK_SIZE};
use crate::layout::{Chunk, LayoutResult};
use crate::resolver::ResolvedInfo;
use crate::schema::{SchemaLayout, TableType};
use crate::utils;
use gptman::{GPTPartitionEntry, GPT};
use mbrman::{MBRPartitionEntry, BOOT_INACTIVE, CHS, MBR};
use std::io::{Seek, SeekFrom};

pub fn generate(
    schema: &SchemaLayout,
    resolved: ResolvedInfo,
) -> Result<LayoutResult, String> {
    let sector_size = schema.sector;
    let mut disk = SparseDisk::new(resolved.disk_size);

    match schema.table {
        TableType::Gpt => {
            let disk_guid = match &schema.disk_id {
                Some(id) => utils::parse_uuid_to_gpt_guid(id)?,
                None => utils::generate_random_guid(),
            };

            let mut gpt = GPT::new_from(&mut disk, sector_size, disk_guid)
                .map_err(|e| e.to_string())?;
            let mut gpt_entry_index = 1;

            for item in &schema.partitions {
                if item.hidden.unwrap_or(false) {
                    continue;
                }

                let p_type = item.partition_type.as_deref().unwrap_or("");
                let guid = utils::parse_uuid_to_gpt_guid(p_type)?;
                let unique_guid = utils::generate_random_guid();

                let rp = resolved.partitions.iter().find(|r| r.id == item.id).unwrap();

                let start_lba = rp.offset / sector_size;
                let end_lba = start_lba + (rp.size / sector_size) - 1;

                gpt[gpt_entry_index] = GPTPartitionEntry {
                    partition_type_guid: guid,
                    unique_partition_guid: unique_guid,
                    starting_lba: start_lba,
                    ending_lba: end_lba,
                    attribute_bits: 0,
                    partition_name: item.id.as_str().into(),
                };
                gpt_entry_index += 1;
            }

            disk.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            GPT::write_protective_mbr_into(&mut disk, sector_size).map_err(|e| e.to_string())?;
            gpt.write_into(&mut disk).map_err(|e| e.to_string())?;
        }
        TableType::Dos => {
            let disk_id_num = match &schema.disk_id {
                Some(id) => u32::from_str_radix(id.trim_start_matches("0x"), 16)
                    .map_err(|e| format!("Invalid DOS ID: {}", e))?
                    .to_le_bytes(),
                None => utils::generate_random_dos_id(),
            };

            let mut mbr = MBR::new_from(&mut disk, sector_size as u32, disk_id_num)
                .map_err(|e| e.to_string())?;

            let mut part_idx = 1;
            for item in &schema.partitions {
                if item.hidden.unwrap_or(false) {
                    continue;
                }

                let rp = resolved.partitions.iter().find(|r| r.id == item.id).unwrap();
                let offset_lba = (rp.offset / sector_size) as u32;
                let size_lba = (rp.size / sector_size) as u32;

                let p_type = item.partition_type.as_deref().unwrap_or("0x83");
                let p_type_byte = u8::from_str_radix(p_type.trim_start_matches("0x"), 16).unwrap();

                mbr[part_idx] = MBRPartitionEntry {
                    boot: BOOT_INACTIVE,
                    starting_lba: offset_lba,
                    sectors: size_lba,
                    sys: p_type_byte,
                    first_chs: CHS::empty(),
                    last_chs: CHS::empty(),
                };
                part_idx += 1;
            }

            disk.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            mbr.write_into(&mut disk).map_err(|e| e.to_string())?;
        }
        TableType::None => {}
    }

    let mut chunks: Vec<Chunk> = Vec::new();

    // skip zero-filled blocks at the start and end of the disk
    for (&block_idx, block_data) in disk.blocks.iter() {
        if let Some(start) = block_data.iter().position(|&b| b != 0) {
            // find the last non-zero byte and include it in the chunk
            let end = block_data.iter().rposition(|&b| b != 0).unwrap() + 1;

            chunks.push(Chunk {
                offset: (block_idx * BLOCK_SIZE) + start as u64,
                data: block_data[start..end].to_vec(),
            });
        }
    }

    Ok(LayoutResult {
        disk_size: resolved.disk_size,
        metadata_chunks: chunks,
        resolved_partitions: resolved.partitions,
    })
}
