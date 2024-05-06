use super::HashKind;
use crate::error::AppError;
use murmur3::murmur3_x64_128;
use sha2::{Digest, Sha256};
use std::{
    fmt::Debug,
    fs::{self, File},
    path::Path,
    time::Instant,
};
use tracing::{info, instrument};
use xxhash_rust::xxh3::xxh3_64;

/// hash a binary file
#[instrument]
pub(super) fn hash_file<T: AsRef<Path> + Debug>(
    file: T,
    kind: HashKind,
) -> Result<String, AppError> {
    let now = Instant::now();
    let hash = match kind {
        HashKind::Sha256 => get_hash_sha256(&file),
        HashKind::Murmur => get_hash_murmur_64(&file),
        HashKind::XxHash => get_hash_xx(&file),
    };
    let elapsed = now.elapsed().as_millis();
    info!("{elapsed} ms - {} s", (elapsed as f64) / 1000.0);
    hash
}

fn get_hash_sha256<T: AsRef<Path> + Debug>(file: T) -> Result<String, AppError> {
    let mut hasher = Sha256::new();
    let bytes = fs::read(file)?;
    hasher.update(bytes);
    let res = hasher.finalize();
    let hash_16 = base16ct::lower::encode_string(&res);
    Ok(hash_16)
}

fn get_hash_murmur_64<T: AsRef<Path> + Debug>(file: T) -> Result<String, AppError> {
    // let mut hasher = Sha256::new();
    // let bytes = fs::read(file)?;
    let mut file = File::open(file).unwrap();
    let res = murmur3_x64_128(&mut file, 0).unwrap();
    Ok(res.to_string())
}

fn get_hash_xx<T: AsRef<Path> + Debug>(file: T) -> Result<String, AppError> {
    let bytes = fs::read(file)?;
    let hash = xxh3_64(&bytes);
    Ok(hash.to_string())
}
