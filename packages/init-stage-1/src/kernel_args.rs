use std::collections::BTreeMap;

use crate::proc::Proc;
#[derive(Debug, Clone)]
pub struct Args {
    pub user_data_partition: String,
    pub system_data_partition: String,
}

impl Args {
    pub fn parse(proc: &Proc) -> std::io::Result<Self> {
        let contents = proc.read_cmdline()?;
        let map = contents
            .split_whitespace()
            .filter_map(|s| s.split_once("="))
            .collect::<BTreeMap<&str, &str>>();
        Ok(Self {
            system_data_partition: map["system_partition"].to_string(),
            user_data_partition: map["user_partition"].to_string(),
        })
    }
}
