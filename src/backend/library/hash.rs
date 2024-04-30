use crate::error::AppError;
use murmur3::murmur3_x64_128;
use sha2::{Digest, Sha256};
use std::{
    fmt::Debug,
    fs::{self, File},
    path::Path,
};
use strum::Display;
use tracing::info;

#[allow(dead_code)]
#[derive(Debug, Display)]
pub(super) enum HashKind {
    Sha256,
    Murmur,
}

/// hash a binary file
pub(super) fn hash_file<T: AsRef<Path> + Debug>(
    file: T,
    kind: HashKind,
) -> Result<String, AppError> {
    info!("hashing {:?} using {kind} started", file);
    let hash = match kind {
        HashKind::Sha256 => get_hash_sha256(&file),
        HashKind::Murmur => get_hash_murmur_64(&file),
    };
    info!("hashed {:?} using {kind} done: {:?}", file, hash);
    hash
}

/// create a SHA-2 of a file
// 197993033
pub fn get_hash_sha256<T: AsRef<Path> + Debug>(file: T) -> Result<String, AppError> {
    let mut hasher = Sha256::new();
    let bytes = fs::read(file)?;
    hasher.update(bytes);
    let res = hasher.finalize();
    let hash_16 = base16ct::lower::encode_string(&res);
    Ok(hash_16)
}

// 171616744
pub fn get_hash_murmur_64<T: AsRef<Path> + Debug>(file: T) -> Result<String, AppError> {
    // let mut hasher = Sha256::new();
    // let bytes = fs::read(file)?;
    let mut file = File::open(file).unwrap();
    let res = murmur3_x64_128(&mut file, 0).unwrap();
    Ok(res.to_string())
}
