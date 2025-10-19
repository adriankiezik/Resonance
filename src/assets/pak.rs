use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::path::Path;
use thiserror::Error;

const PAK_MAGIC: &[u8; 6] = b"RESPAK";
const PAK_VERSION: u32 = 1;
const MAX_ASSET_SIZE: u64 = 1024 * 1024 * 1024;

#[derive(Error, Debug)]
pub enum PakError {
    #[error("Invalid PAK magic header")]
    InvalidMagic,
    #[error("Unsupported PAK version: {0}")]
    UnsupportedVersion(u32),
    #[error("Asset not found in PAK: {0}")]
    AssetNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Failed to compress data: {0}")]
    CompressionFailed(String),
    #[error("Failed to decompress data: {0}")]
    DecompressionFailed(String),
}

#[derive(Debug, Clone)]
pub struct PakEntry {
    pub path: String,
    pub offset: u64,
    pub size: u64,
    pub checksum: u32,
    pub compressed: bool,
    pub original_size: u64,
}

impl PakEntry {
    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let path_bytes = self.path.as_bytes();
        writer.write_all(&(path_bytes.len() as u32).to_le_bytes())?;
        writer.write_all(path_bytes)?;

        writer.write_all(&self.offset.to_le_bytes())?;
        writer.write_all(&self.size.to_le_bytes())?;
        writer.write_all(&self.checksum.to_le_bytes())?;
        writer.write_all(&(self.compressed as u8).to_le_bytes())?;
        writer.write_all(&self.original_size.to_le_bytes())?;

        Ok(())
    }

    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut path_len_bytes = [0u8; 4];
        reader.read_exact(&mut path_len_bytes)?;
        let path_len = u32::from_le_bytes(path_len_bytes) as usize;

        let mut path_bytes = vec![0u8; path_len];
        reader.read_exact(&mut path_bytes)?;
        let path = String::from_utf8(path_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut offset_bytes = [0u8; 8];
        reader.read_exact(&mut offset_bytes)?;
        let offset = u64::from_le_bytes(offset_bytes);

        let mut size_bytes = [0u8; 8];
        reader.read_exact(&mut size_bytes)?;
        let size = u64::from_le_bytes(size_bytes);

        let mut checksum_bytes = [0u8; 4];
        reader.read_exact(&mut checksum_bytes)?;
        let checksum = u32::from_le_bytes(checksum_bytes);

        let mut compressed_bytes = [0u8; 1];
        reader.read_exact(&mut compressed_bytes)?;
        let compressed = compressed_bytes[0] != 0;

        let mut original_size_bytes = [0u8; 8];
        reader.read_exact(&mut original_size_bytes)?;
        let original_size = u64::from_le_bytes(original_size_bytes);

        Ok(PakEntry {
            path,
            offset,
            size,
            checksum,
            compressed,
            original_size,
        })
    }
}

pub struct PakArchive {
    data: Vec<u8>,
    entries: HashMap<String, PakEntry>,
}

impl PakArchive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, PakError> {
        let data = std::fs::read(path.as_ref())?;
        Self::from_bytes(data)
    }

    pub fn from_bytes(data: Vec<u8>) -> Result<Self, PakError> {
        let mut cursor = io::Cursor::new(&data);

        let mut magic = [0u8; 6];
        cursor.read_exact(&mut magic)?;
        if &magic != PAK_MAGIC {
            return Err(PakError::InvalidMagic);
        }

        let mut version_bytes = [0u8; 4];
        cursor.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);
        if version != PAK_VERSION {
            return Err(PakError::UnsupportedVersion(version));
        }

        let mut count_bytes = [0u8; 4];
        cursor.read_exact(&mut count_bytes)?;
        let file_count = u32::from_le_bytes(count_bytes);

        let mut entries = HashMap::new();
        for _ in 0..file_count {
            let entry = PakEntry::read(&mut cursor)?;
            entries.insert(entry.path.clone(), entry);
        }

        Ok(Self { data, entries })
    }

    pub fn get(&self, path: &str) -> Result<Vec<u8>, PakError> {
        let entry = self
            .entries
            .get(path)
            .ok_or_else(|| PakError::AssetNotFound(path.to_string()))?;

        if entry.original_size > MAX_ASSET_SIZE {
            return Err(PakError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Asset {} exceeds maximum size ({} > {} bytes)",
                    path, entry.original_size, MAX_ASSET_SIZE
                ),
            )));
        }

        let start = entry.offset as usize;
        let end = start + entry.size as usize;

        if end > self.data.len() {
            return Err(PakError::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Entry offset/size exceeds PAK file size",
            )));
        }

        let mut data = self.data[start..end].to_vec();

        if entry.compressed {
            data = decompress_data(&data)?;
        }

        let checksum = crc32fast::hash(&data);
        if checksum != entry.checksum {
            log::warn!(
                "Checksum mismatch for {}: expected {}, got {}",
                path,
                entry.checksum,
                checksum
            );
        }

        Ok(data)
    }

    pub fn exists(&self, path: &str) -> bool {
        self.entries.contains_key(path)
    }

    pub fn list(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    pub fn get_entry(&self, path: &str) -> Option<&PakEntry> {
        self.entries.get(path)
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

pub struct PakBuilder {
    entries: Vec<(String, Vec<u8>)>,
    compress: bool,
}

impl PakBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            compress: false,
        }
    }

    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    pub fn add_file(&mut self, pak_path: String, file_path: impl AsRef<Path>) -> io::Result<()> {
        let data = std::fs::read(file_path.as_ref())?;
        self.add_bytes(pak_path, data);
        Ok(())
    }

    pub fn add_bytes(&mut self, pak_path: String, data: Vec<u8>) {
        self.entries.push((pak_path, data));
    }

    pub fn build(&self, output_path: impl AsRef<Path>) -> Result<(), PakError> {
        let data = self.build_to_bytes()?;
        std::fs::write(output_path.as_ref(), data)?;
        Ok(())
    }

    pub fn build_to_bytes(&self) -> Result<Vec<u8>, PakError> {
        let mut buffer = Vec::new();

        buffer.write_all(PAK_MAGIC)?;
        buffer.write_all(&PAK_VERSION.to_le_bytes())?;
        buffer.write_all(&(self.entries.len() as u32).to_le_bytes())?;

        let mut data_offset = buffer.len() as u64;
        for (path, _) in &self.entries {
            data_offset += 4 + path.len() as u64 + 8 + 8 + 4 + 1 + 8;
        }

        let mut pak_entries = Vec::new();
        let mut data_buffer = Vec::new();

        for (path, data) in &self.entries {
            let original_size = data.len() as u64;
            let checksum = crc32fast::hash(data);

            let (final_data, compressed) = if self.compress {
                match compress_data(data) {
                    Ok(compressed_data) => {
                        if compressed_data.len() < data.len() {
                            (compressed_data, true)
                        } else {
                            (data.clone(), false)
                        }
                    }
                    Err(_) => (data.clone(), false),
                }
            } else {
                (data.clone(), false)
            };

            let entry = PakEntry {
                path: path.clone(),
                offset: data_offset + data_buffer.len() as u64,
                size: final_data.len() as u64,
                checksum,
                compressed,
                original_size,
            };

            pak_entries.push(entry);
            data_buffer.extend_from_slice(&final_data);
        }

        for entry in &pak_entries {
            entry.write(&mut buffer)?;
        }

        buffer.extend_from_slice(&data_buffer);

        Ok(buffer)
    }
}

impl Default for PakBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn compress_data(data: &[u8]) -> Result<Vec<u8>, PakError> {
    use flate2::Compression;
    use flate2::write::DeflateEncoder;

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(data)
        .map_err(|e| PakError::CompressionFailed(e.to_string()))?;
    encoder
        .finish()
        .map_err(|e| PakError::CompressionFailed(e.to_string()))
}

fn decompress_data(data: &[u8]) -> Result<Vec<u8>, PakError> {
    use flate2::read::DeflateDecoder;

    let mut decoder = DeflateDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| PakError::DecompressionFailed(e.to_string()))?;
    Ok(decompressed)
}
