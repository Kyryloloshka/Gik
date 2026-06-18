use crate::core::hash::Hash;
use crate::core::models::CommitMeta;
use crate::core::pack::utils::{apply_delta, read_object_header};
use crate::core::storage::Storage;
use crate::error::{GikError, Result};
use flate2::bufread::ZlibDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};

struct PendingDelta {
    base_hash: Hash,
    delta_data: Vec<u8>,
    offset: u64,
}

struct PositionTracker<R> {
    inner: R,
    pos: u64,
}

impl<R: Read> Read for PositionTracker<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl<R: std::io::BufRead> std::io::BufRead for PositionTracker<R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.inner.fill_buf()
    }
    fn consume(&mut self, amt: usize) {
        self.pos += amt as u64;
        self.inner.consume(amt);
    }
}

fn get_base_payload(
    hash: &Hash,
    current_pack_index: &HashMap<Hash, u64>,
    pack_path: &std::path::Path,
    storage: &Storage,
) -> Result<Option<(String, Vec<u8>)>> {
    if let Some(&offset) = current_pack_index.get(hash) {
        let mut file = std::fs::File::open(pack_path).map_err(|e| GikError::Io(e))?;
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| GikError::Io(e))?;
        let mut buf_reader = std::io::BufReader::new(file);
        let (obj_type, _) = read_object_header(&mut buf_reader)?;

        if obj_type == 7 {
            // OBJ_REF_DELTA
            let mut base_hash_bytes = [0u8; 20];
            buf_reader
                .read_exact(&mut base_hash_bytes)
                .map_err(|e| GikError::Io(e))?;
            let base_hash = Hash(base_hash_bytes);

            let mut zlib = ZlibDecoder::new(&mut buf_reader);
            let mut delta_data = Vec::new();
            zlib.read_to_end(&mut delta_data)
                .map_err(|e| GikError::Io(e))?;

            let (type_str, base_payload) =
                get_base_payload(&base_hash, current_pack_index, pack_path, storage)?.ok_or_else(
                    || {
                        GikError::Io(std::io::Error::other(format!(
                            "Missing base for delta: {}",
                            base_hash
                        )))
                    },
                )?;

            let target_payload = apply_delta(&base_payload, &delta_data)?;
            return Ok(Some((type_str, target_payload)));
        } else if obj_type == 6 {
            return Err(GikError::Io(std::io::Error::other(
                "OBJ_OFS_DELTA not supported yet",
            )));
        } else {
            let type_str = match obj_type {
                1 => "commit",
                2 => "tree",
                3 => "blob",
                _ => {
                    return Err(GikError::Io(std::io::Error::other(format!(
                        "Unsupported pack object type: {}",
                        obj_type
                    ))))
                }
            };
            let mut zlib = ZlibDecoder::new(&mut buf_reader);
            let mut decompressed = Vec::new();
            zlib.read_to_end(&mut decompressed)
                .map_err(|e| GikError::Io(e))?;
            return Ok(Some((type_str.to_string(), decompressed)));
        }
    } else {
        if let Ok(Some(compressed_base)) = storage.objects().get_object(hash) {
            let (type_str, _, base_payload) =
                crate::core::objects::decompress_object(&compressed_base[..])
                    .map_err(|e| GikError::Io(e))?;
            return Ok(Some((type_str, base_payload)));
        }
        return Ok(None);
    }
}

pub fn decode_packfile<R: Read>(reader_in: &mut R, storage: &Storage) -> Result<()> {
    // 1. Save stream to disk
    let mut header = [0u8; 12];
    reader_in
        .read_exact(&mut header)
        .map_err(|e| GikError::Io(e))?;
    if &header[0..4] != b"PACK" {
        return Err(GikError::Io(std::io::Error::other(format!(
            "Invalid packfile signature: {:?}",
            String::from_utf8_lossy(&header)
        ))));
    }

    let pack_dir = storage.objects_dir.join("pack");
    std::fs::create_dir_all(&pack_dir).map_err(|e| GikError::Io(e))?;

    let pack_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let pack_name = format!("pack-{}.pack", pack_id);
    let pack_path = pack_dir.join(&pack_name);

    let pb_download = ProgressBar::new_spinner();
    pb_download.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] Downloading packfile... {bytes} ({bytes_per_sec})")
            .unwrap(),
    );
    let mut wrapped_reader = pb_download.wrap_read(reader_in);

    let mut file = std::fs::File::create(&pack_path).map_err(|e| GikError::Io(e))?;
    file.write_all(&header).map_err(|e| GikError::Io(e))?;
    std::io::copy(&mut wrapped_reader, &mut file).map_err(|e| GikError::Io(e))?;
    file.sync_all().map_err(|e| GikError::Io(e))?;

    pb_download.finish_with_message("Download completed.");

    // 2. Parse and index
    let file = std::fs::File::open(&pack_path).map_err(|e| GikError::Io(e))?;
    let mut tracker = PositionTracker {
        inner: std::io::BufReader::new(file),
        pos: 0,
    };
    tracker
        .read_exact(&mut header)
        .map_err(|e| GikError::Io(e))?;

    let object_count = u32::from_be_bytes(header[8..12].try_into().unwrap());
    let pb = ProgressBar::new(object_count as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));
    pb.set_message("Indexing packfile...");

    let mut pending_deltas = Vec::new();
    let mut current_pack_index = HashMap::new(); // Hash -> offset
    let mut commit_metas = Vec::new();

    for _i in 0..object_count {
        pb.inc(1);
        let offset = tracker.pos;

        let (obj_type, _size) = read_object_header(&mut tracker)?;

        if obj_type == 7 {
            // OBJ_REF_DELTA
            let mut base_hash_bytes = [0u8; 20];
            tracker
                .read_exact(&mut base_hash_bytes)
                .map_err(|e| GikError::Io(e))?;
            let base_hash = Hash(base_hash_bytes);

            let mut zlib = ZlibDecoder::new(&mut tracker);
            let mut delta_data = Vec::new();
            zlib.read_to_end(&mut delta_data)
                .map_err(|e| GikError::Io(e))?;

            pending_deltas.push(PendingDelta {
                base_hash,
                delta_data,
                offset,
            });
            continue;
        } else if obj_type == 6 {
            return Err(GikError::Io(std::io::Error::other(
                "OBJ_OFS_DELTA not supported yet",
            )));
        }

        let type_str = match obj_type {
            1 => "commit",
            2 => "tree",
            3 => "blob",
            _ => {
                return Err(GikError::Io(std::io::Error::other(format!(
                    "Unsupported pack object type: {}",
                    obj_type
                ))))
            }
        };

        let mut zlib = ZlibDecoder::new(&mut tracker);
        let mut decompressed = Vec::new();
        zlib.read_to_end(&mut decompressed)
            .map_err(|e| GikError::Io(e))?;

        let header_str = format!("{} {}\0", type_str, decompressed.len());
        let mut full_obj = header_str.into_bytes();
        full_obj.extend_from_slice(&decompressed);

        let mut hasher = Sha1::new();
        hasher.update(&full_obj);
        let hash_bytes: [u8; 20] = hasher.finalize().into();
        let hash = Hash::from(hash_bytes);

        current_pack_index.insert(hash.clone(), offset);

        if obj_type == 1 {
            let meta = parse_commit_meta(&decompressed)?;
            commit_metas.push((hash, meta));
        }
    }

    pb.finish_with_message("Indexing completed.");

    let pb_deltas = ProgressBar::new(pending_deltas.len() as u64);
    pb_deltas.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.magenta/red}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));
    pb_deltas.set_message("Resolving deltas...");

    // Resolve deltas
    while !pending_deltas.is_empty() {
        let mut resolved_any = false;
        let mut next_pending = Vec::new();

        for delta in pending_deltas {
            let base_payload_opt =
                get_base_payload(&delta.base_hash, &current_pack_index, &pack_path, storage)?;

            if let Some((type_str, base_payload)) = base_payload_opt {
                let target_payload = apply_delta(&base_payload, &delta.delta_data)?;

                let header_str = format!("{} {}\0", type_str, target_payload.len());
                let mut full_obj = header_str.into_bytes();
                full_obj.extend_from_slice(&target_payload);

                let mut hasher = Sha1::new();
                hasher.update(&full_obj);
                let hash_bytes: [u8; 20] = hasher.finalize().into();
                let hash = Hash::from(hash_bytes);

                current_pack_index.insert(hash.clone(), delta.offset);

                if type_str == "commit" {
                    let meta = parse_commit_meta(&target_payload)?;
                    commit_metas.push((hash, meta));
                }

                resolved_any = true;
                pb_deltas.inc(1);
            } else {
                next_pending.push(delta);
            }
        }

        if !resolved_any && !next_pending.is_empty() {
            return Err(GikError::Io(std::io::Error::other(format!(
                "Missing base object for delta: {}",
                next_pending[0].base_hash
            ))));
        }
        pb_deltas.set_length((pb_deltas.position() + next_pending.len() as u64) as u64);
        pending_deltas = next_pending;
    }

    pb_deltas.finish_with_message("Deltas resolved.");

    // 3. Write index to Redb
    let write_txn = storage.repo.db.begin_write()?;
    {
        let mut pack_table = write_txn.open_table(crate::core::storage::repository::PACKFILES)?;
        pack_table.insert(pack_id, pack_name.as_str())?;

        let mut index_table =
            write_txn.open_table(crate::core::storage::repository::PACKFILE_INDEX)?;

        for (hash, offset) in current_pack_index {
            index_table.insert(&hash.0, (pack_id, offset))?;
        }
    }
    write_txn.commit()?;

    // Save commit metas
    for (hash, meta) in commit_metas {
        storage.commits().insert_commit_meta(&hash, meta)?;
    }

    Ok(())
}

fn parse_commit_meta(content: &[u8]) -> Result<CommitMeta> {
    let text = String::from_utf8_lossy(content);
    let mut tree_hash = Hash([0; 20]);
    let mut parent_hashes = Vec::new();
    let mut author = String::new();
    let mut timestamp = 0;

    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.is_empty() {
            break;
        }
        if let Some(rest) = line.strip_prefix("tree ") {
            tree_hash = Hash::from_hex(rest).map_err(|e| GikError::Io(std::io::Error::other(e)))?;
        } else if let Some(rest) = line.strip_prefix("parent ") {
            parent_hashes
                .push(Hash::from_hex(rest).map_err(|e| GikError::Io(std::io::Error::other(e)))?);
        } else if let Some(rest) = line.strip_prefix("author ") {
            if let Some(tz_idx) = rest.rfind(" +") {
                if let Some(ts_idx) = rest[..tz_idx].rfind(' ') {
                    author = rest[..ts_idx].to_string();
                    timestamp = rest[ts_idx + 1..tz_idx].parse().unwrap_or(0);
                }
            }
        }
    }

    let message = lines.collect::<Vec<_>>().join("\n");

    Ok(CommitMeta {
        tree_hash,
        parent_hashes,
        author,
        timestamp,
        message,
    })
}
