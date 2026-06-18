use crate::error::Result;
use crate::core::hash::Hash;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::fs;

pub struct ObjectService<'a> {
    pub(crate) objects_dir: &'a Path,
}

impl<'a> ObjectService<'a> {
    fn get_object_path(&self, hash: &Hash) -> PathBuf {
        let hash_str = hash.to_string();
        self.objects_dir.join(&hash_str[0..2]).join(&hash_str[2..])
    }

    pub fn contains_object(&self, hash: &Hash) -> Result<bool> {
        Ok(self.get_object_path(hash).exists())
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

        let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let tmp_path = parent.join(format!("{}{}_{}", crate::config::TMP_OBJECT_PREFIX, hash.to_string(), time));
        
        {
            let mut file = fs::File::create(&tmp_path).map_err(|e| crate::error::GikError::Io(e))?;
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
            if entry.file_type().map_err(|e| crate::error::GikError::Io(e))?.is_dir() {
                let prefix = entry.file_name().to_string_lossy().to_string();
                if prefix.len() == 2 {
                    for sub_entry in fs::read_dir(entry.path()).map_err(|e| crate::error::GikError::Io(e))? {
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
            file.read_to_end(&mut data).map_err(|e| crate::error::GikError::Io(e))?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    pub fn get_blob_text(&self, hash: &Hash) -> Result<String> {
        if let Some(compressed_data) = self.get_object(hash)? {
            let (obj_type, _size, content) = crate::core::objects::decompress_object(&compressed_data[..])?;
            if obj_type != "blob" {
                return Err(crate::error::GikError::Validation(format!("Object {} is not a blob (type: {})", hash, obj_type)));
            }
            Ok(String::from_utf8_lossy(&content).to_string())
        } else {
            return Err(crate::error::GikError::NotFound(format!("Loose object {} not found", hash)));
        }
    }
}
