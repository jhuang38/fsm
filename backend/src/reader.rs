use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::BufReader,
    path::{Path, PathBuf},
};

extern crate serde;
extern crate serde_derive;
extern crate serde_json;


use serde::{Deserialize, Serialize};

use crate::{error::FsmError, filter::FileFilter};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum DirectoryEntry {
    LeafDirectory(String),
    ParentDirectory(HashMap<String, DirectoryEntry>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FsmConfigRepresentation {
    pub watch_path: String,
    pub managed_path: String,
    pub managed_directory_structure: DirectoryEntry,
    pub filters: Vec<FileFilter>,
    pub overwrite_on_move: bool,
    pub sweep_loop_time: u64, // representation in minutes, may switch to more user-friendly format
}

pub fn read_fsm_config<P>(file_path: P) -> Result<FsmConfigRepresentation, FsmError>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let result: FsmConfigRepresentation = serde_json::from_reader(&mut reader)?;
    Ok(result)
}
