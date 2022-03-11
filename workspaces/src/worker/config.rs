use std::{path::Path, io::BufReader, fs::File};

use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub genesis_time: u64,
    // pub gas_price: Balance,
    // pub gas_limit: Gas,
    pub genesis_height: u64,
    pub epoch_length: u64,
    // pub block_prod_time: Duration,
    // pub runtime_config: RuntimeConfig,
    // pub state_records: Vec<StateRecord>,
    // pub validators: Vec<AccountInfo>,
}

// impl Default for GenesisConfig {
//     fn default() -> Self {
//         Self {
//             genesis_time: ,
//             gas_price: ,
//             gas_limit: ,
//             genesis_height: ,
//             epoch_length: ,
//             block_prod_time: ,
//             runtime_config: ,
//             state_records: ,
//             validators: ,
//         }
//     }
// }


impl GenesisConfig {
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let reader = BufReader::new(File::open(path)?);
        let genesis: Self = serde_json::from_reader(reader)?;
        Ok(genesis)
    }
}
