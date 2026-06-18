use crate::core::hash::Hash;
use crate::error::Result;
use redb::ReadableTable;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::core::storage::repository::Repository;

pub struct ObjectService<'a> {
    pub(crate) objects_dir: &'a Path,
    pub(crate) repo: &'a Repository,
}

impl<'a> ObjectService<'a> {
    fn get_object_path(&self, hash: &Hash) -> PathBuf {
        let hash_str = hash.to_string();
        self.objects_dir.join(&hash_str[0..2]).join(&hash_str[2..])
    }

    pub fn contains_object(&self, hash: &Hash) -> Result<bool> {
        if self.get_object_path(hash).exists() {
            return Ok(true);
        }

        let read_txn = self.repo.db.begin_read()?;
        let index_table = read_txn.open_table(crate::core::storage::repository::PACKFILE_INDEX)?;

        let exists = index_table.get(&hash.0)?.is_some();
        Ok(exists)
    }

    pub fn write_object_with_writer<F>(&self, hash: &Hash, writer_fn: F) -> Result<()>
    where
        F: FnOnce(&mut fs::File) -> Result<()>,
    {
        let path = self.get_object_path(hash);
        if path.exists() {
            return Ok(()); // Already exists
        }

        let parent = path.parent().unwrap();
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| crate::error::GikError::Io(e))?;
        }

        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_path = parent.join(format!(
            "{}{}_{}",
            crate::config::TMP_OBJECT_PREFIX,
            hash.to_string(),
            time
        ));

        {
            let mut file =
                fs::File::create(&tmp_path).map_err(|e| crate::error::GikError::Io(e))?;
            writer_fn(&mut file)?;
            file.sync_all().map_err(|e| crate::error::GikError::Io(e))?;
        }

        // Atomic rename
        fs::rename(&tmp_path, &path).map_err(|e| crate::error::GikError::Io(e))?;
        Ok(())
    }

    pub fn write_object_stream<R: Read>(&self, hash: &Hash, mut reader: R) -> Result<()> {
        self.write_object_with_writer(hash, |file| {
            std::io::copy(&mut reader, file).map_err(|e| crate::error::GikError::Io(e))?;
            Ok(())
        })
    }

    pub fn write_object(&self, hash: &Hash, compressed_data: &[u8]) -> Result<()> {
        self.write_object_stream(hash, compressed_data)
    }

    pub fn list_all_objects(&self) -> Result<Vec<Hash>> {
        let mut hashes = Vec::new();
        if !self.objects_dir.exists() {
            return Ok(hashes);
        }
        for entry in fs::read_dir(self.objects_dir).map_err(|e| crate::error::GikError::Io(e))? {
            let entry = entry.map_err(|e| crate::error::GikError::Io(e))?;
            if entry
                .file_type()
                .map_err(|e| crate::error::GikError::Io(e))?
                .is_dir()
            {
                let prefix = entry.file_name().to_string_lossy().to_string();
                if prefix.len() == 2 {
                    for sub_entry in
                        fs::read_dir(entry.path()).map_err(|e| crate::error::GikError::Io(e))?
                    {
                        let sub_entry = sub_entry.map_err(|e| crate::error::GikError::Io(e))?;
                        let suffix = sub_entry.file_name().to_string_lossy().to_string();
                        if let Ok(hash) = Hash::from_hex(&(prefix.clone() + &suffix)) {
                            hashes.push(hash);
                        }
                    }
                }
            }
        }
        Ok(hashes)
    }

    pub fn get_object_stream(&self, hash: &Hash) -> Result<Option<fs::File>> {
        let path = self.get_object_path(hash);
        if path.exists() {
            let file = fs::File::open(path).map_err(|e| crate::error::GikError::Io(e))?;
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }

    pub fn get_object(&self, hash: &Hash) -> Result<Option<Vec<u8>>> {
        if let Some(mut file) = self.get_object_stream(hash)? {
            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .map_err(|e| crate::error::GikError::Io(e))?;
            return Ok(Some(data));
        }

        let (pack_id, offset) = {
            let read_txn = self.repo.db.begin_read()?;
            let index_table =
                read_txn.open_table(crate::core::storage::repository::PACKFILE_INDEX)?;
            let val = index_table.get(&hash.0)?.map(|e| e.value());
            if let Some(v) = val {
                v
            } else {
                return Ok(None);
            }
        };

        let pack_name_str = {
            let read_txn = self.repo.db.begin_read()?;
            let packfiles_table =
                read_txn.open_table(crate::core::storage::repository::PACKFILES)?;
            let val = packfiles_table.get(pack_id)?.map(|e| e.value().to_string());
            val.ok_or_else(|| {
                crate::error::GikError::NotFound(format!("Pack ID {} not found", pack_id))
            })?
        };

        return self.read_from_packfile(&pack_name_str, offset);
    }

    fn read_from_packfile(&self, pack_name: &str, offset: u64) -> Result<Option<Vec<u8>>> {
        use flate2::bufread::ZlibDecoder;
        use std::io::{BufReader, Seek, SeekFrom};

        let pack_path = self.objects_dir.join("pack").join(pack_name);
        let mut file = fs::File::open(pack_path).map_err(|e| crate::error::GikError::Io(e))?;
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| crate::error::GikError::Io(e))?;

        let mut reader = BufReader::new(file);
        let (obj_type, _size) = crate::core::pack::utils::read_object_header(&mut reader)?;

        if obj_type == 7 {
            // OBJ_REF_DELTA
            let mut base_hash_bytes = [0u8; 20];
            reader
                .read_exact(&mut base_hash_bytes)
                .map_err(|e| crate::error::GikError::Io(e))?;
            let base_hash = Hash(base_hash_bytes);

            let mut zlib = ZlibDecoder::new(&mut reader);
            let mut delta_data = Vec::new();
            zlib.read_to_end(&mut delta_data)
                .map_err(|e| crate::error::GikError::Io(e))?;

            let compressed_base = self.get_object(&base_hash)?.ok_or_else(|| {
                crate::error::GikError::NotFound(format!("Missing base object {}", base_hash))
            })?;

            let (type_str, _, base_payload) =
                crate::core::objects::decompress_object(&compressed_base[..])?;

            let target_payload = crate::core::pack::utils::apply_delta(&base_payload, &delta_data)?;

            let header_str = format!("{} {}\0", type_str, target_payload.len());
            let mut full_obj = header_str.into_bytes();
            full_obj.extend_from_slice(&target_payload);

            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            std::io::Write::write_all(&mut encoder, &full_obj)
                .map_err(|e| crate::error::GikError::Io(e))?;
            return Ok(Some(
                encoder
                    .finish()
                    .map_err(|e| crate::error::GikError::Io(e))?,
            ));
        } else if obj_type == 1 || obj_type == 2 || obj_type == 3 {
            let type_str = match obj_type {
                1 => "commit",
                2 => "tree",
                3 => "blob",
                _ => unreachable!(),
            };

            let mut zlib = ZlibDecoder::new(&mut reader);
            let mut decompressed = Vec::new();
            zlib.read_to_end(&mut decompressed)
                .map_err(|e| crate::error::GikError::Io(e))?;

            let header_str = format!("{} {}\0", type_str, decompressed.len());
            let mut full_obj = header_str.into_bytes();
            full_obj.extend_from_slice(&decompressed);

            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            std::io::Write::write_all(&mut encoder, &full_obj)
                .map_err(|e| crate::error::GikError::Io(e))?;
            return Ok(Some(
                encoder
                    .finish()
                    .map_err(|e| crate::error::GikError::Io(e))?,
            ));
        } else {
            return Err(crate::error::GikError::Validation(format!(
                "Unsupported pack object type {}",
                obj_type
            )));
        }
    }

    pub fn get_blob_text(&self, hash: &Hash) -> Result<String> {
        if let Some(compressed_data) = self.get_object(hash)? {
            let (obj_type, _size, content) =
                crate::core::objects::decompress_object(&compressed_data[..])?;
            if obj_type != "blob" {
                return Err(crate::error::GikError::Validation(format!(
                    "Object {} is not a blob (type: {})",
                    hash, obj_type
                )));
            }
            Ok(String::from_utf8_lossy(&content).to_string())
        } else {
            return Err(crate::error::GikError::NotFound(format!(
                "Loose object {} not found",
                hash
            )));
        }
    }
}
