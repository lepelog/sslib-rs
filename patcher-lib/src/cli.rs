use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub enum Context {
    FullExtract { vanilla_iso: PathBuf, modified_extract_dir: PathBuf,
        #[clap(default_value = "0")]
        free_space_bytes: u32,
        single_stage: Option<String>,
    },
    SingleStage { vanilla_iso: PathBuf, stage: String,
        #[clap(default_value = "0")]
        free_space_bytes: u32
    }
}
