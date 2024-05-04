use crate::{dotfile::DotfileSchema, error::AppError};
use csv::{Reader, Writer};
use std::fs::{File, OpenOptions};

pub fn app_writer_non_append() -> Result<Writer<File>, AppError> {
    let path = DotfileSchema::cache_path()?;
    Ok(csv::WriterBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_path(path)?)
}

pub fn app_reader() -> Result<Reader<File>, AppError> {
    let path = DotfileSchema::cache_path()?;
    Ok(csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_path(path)?)
}

pub fn app_writer_append() -> Result<Writer<File>, AppError> {
    let path = DotfileSchema::cache_path()?;
    let wtr = OpenOptions::new().append(true).open(path)?;

    Ok(csv::WriterBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_writer(wtr))
}
