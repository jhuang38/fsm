use std::{collections::HashMap, fs, io::BufReader, path::Path, time::Duration};

extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use serde::{Deserialize, Serialize};

use crate::{error::FsmError, filter::FileFilter};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum DirectoryEntry {
    LeafDirectory(String),
    ParentDirectory(HashMap<String, DirectoryEntry>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeInterval {
    seconds: Option<u64>,
    minutes: Option<u64>,
    hours: Option<u64>,
}

impl TimeInterval {
    pub fn new(seconds: u64, minutes: u64, hours: u64) -> Self {
        Self {
            seconds: Some(seconds),
            minutes: Some(minutes),
            hours: Some(hours),
        }
    }
}

impl Default for TimeInterval {
    fn default() -> Self {
        Self {
            seconds: Some(60),
            minutes: Some(0),
            hours: Some(0),
        }
    }
}

impl Into<Duration> for TimeInterval {
    fn into(self) -> Duration {
        Duration::from_secs(self.seconds.unwrap_or(0))
            + Duration::from_secs(self.minutes.unwrap_or(0) * 60)
            + Duration::from_secs(self.hours.unwrap_or(0) * 3600)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FsmConfigRepresentation {
    pub watch_path: String,
    pub managed_path: String,
    pub managed_directory_structure: DirectoryEntry,
    pub filters: Vec<FileFilter>,
    pub overwrite_on_move: bool,
    pub sweep_loop_time: Option<TimeInterval>,
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
