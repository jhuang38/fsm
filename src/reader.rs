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

impl FsmConfigRepresentation {
    pub fn new() -> Self {
        Self {
            watch_path: "".to_string(),
            managed_path: "".to_string(),
            managed_directory_structure: DirectoryEntry::LeafDirectory("".to_string()),
            filters: Vec::new(),
            overwrite_on_move: false,
            sweep_loop_time: None,
        }
    }
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

#[cfg(test)]
mod tests {
    use std::io::{BufWriter, Write};

    use super::*;

    struct TestConfig;

    impl TestConfig {
        pub fn new() -> Self {
            _ = fs::File::create("reader_unit_test.json").unwrap();
            Self
        }
    }

    impl Drop for TestConfig {
        fn drop(&mut self) {
            fs::remove_file("reader_unit_test.json").unwrap();
        }
    }

    #[test]
    fn test_read_fsm_config() {
        let cfg = TestConfig::new();

        let mut test_repr = FsmConfigRepresentation::new();
        test_repr.watch_path = "test1".to_string();
        test_repr.managed_path = "test2".to_string();
        let test_dir_repr = DirectoryEntry::ParentDirectory(HashMap::from([
            (
                "testdir1".to_string(),
                DirectoryEntry::LeafDirectory("leafdir1".to_string()),
            ),
            (
                "testdir2".to_string(),
                DirectoryEntry::ParentDirectory(HashMap::from([(
                    "testdir3".to_string(),
                    DirectoryEntry::LeafDirectory("leafdir2".to_string()),
                )])),
            ),
        ]));
        test_repr.managed_directory_structure = test_dir_repr.clone();
        let test_filters = Vec::from([FileFilter::default(), FileFilter::default()]);
        test_repr.filters.append(&mut test_filters.clone());
        test_repr.overwrite_on_move = false;
        test_repr.sweep_loop_time = None;

        let test_file = fs::File::options()
            .read(true)
            .write(true)
            .open("reader_unit_test.json")
            .unwrap();
        let mut writer = BufWriter::new(test_file);
        serde_json::to_writer(&mut writer, &test_repr).unwrap();
        _ = writer.flush();

        let read_repr = read_fsm_config("reader_unit_test.json").unwrap();
        assert_eq!(test_repr.watch_path, read_repr.watch_path);
        assert_eq!(test_repr.managed_path, read_repr.managed_path);
        // TODO: write eq, etc. impls for these
        // assert_eq!(test_repr.managed_directory_structure, read_repr.managed_directory_structure);
        // assert_eq!(test_repr.filters, read_repr.filters);
        assert_eq!(test_repr.overwrite_on_move, read_repr.overwrite_on_move);
        // assert_eq!(test_repr.sweep_loop_time, read_repr.sweep_loop_time);
    }
}
