use uuid::Uuid;

pub fn parse_uuid_to_gpt_guid(uuid_str: &str) -> Result<[u8; 16], String> {
    let id = Uuid::parse_str(uuid_str).map_err(|e| e.to_string())?;
    Ok(id.to_bytes_le())
}

pub fn generate_random_guid() -> [u8; 16] {
    Uuid::new_v4().to_bytes_le()
}

pub fn generate_random_dos_id() -> [u8; 4] {
    let mut buf = [0u8; 4];
    getrandom::getrandom(&mut buf).unwrap(); 
    buf
}
