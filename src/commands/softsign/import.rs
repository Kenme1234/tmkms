//! `tmkms softsign import` command

use crate::{config::provider::softsign::KeyFormat, key_utils, prelude::*};
use abscissa_core::{Command, Options, Runnable};
use std::{path::PathBuf, process};
use tendermint::PrivateKey;
use tendermint_config::PrivValidatorKey;

/// `import` command: import a `priv_validator.json` formatted key and convert
/// it into the raw format used by the softsign backend (by default)
#[derive(Command, Debug, Default, Options)]
pub struct ImportCommand {
    #[options(
        short = "f",
        help = "key format to import: 'json' or 'raw' (default 'json')"
    )]
    format: Option<String>,

    #[options(free, help = "[INPUT] and [OUTPUT] paths for key generation")]
    paths: Vec<PathBuf>,
}

impl Runnable for ImportCommand {
    /// Import a `priv_validator.json`
    fn run(&self) {
        if self.paths.len() != 2 {
            status_err!("expected 2 arguments, got {}", self.paths.len());
            eprintln!("\nUsage: tmkms softsign import [priv_validator.json] [output.key]");
            process::exit(1);
        }

        let input_path = &self.paths[0];
        let output_path = &self.paths[1];

        let format = self
            .format
            .as_ref()
            .map(|f| {
                f.parse::<KeyFormat>().unwrap_or_else(|e| {
                    status_err!("{} (must be 'json' or 'raw')", e);
                    process::exit(1);
                })
            })
            .unwrap_or(KeyFormat::Json);

        if format != KeyFormat::Json {
            status_err!("invalid format: {:?} (must be 'json')", format);
            process::exit(1);
        }

        let private_key = PrivValidatorKey::load_json_file(input_path)
            .unwrap_or_else(|e| {
                status_err!("couldn't load {}: {}", input_path.display(), e);
                process::exit(1);
            })
            .priv_key;

        match private_key {
            PrivateKey::Ed25519(pk) => {
                key_utils::write_base64_secret(output_path, pk.secret.as_bytes()).unwrap_or_else(
                    |e| {
                        status_err!("{}", e);
                        process::exit(1);
                    },
                );
            }
            _ => unreachable!("unsupported priv_validator.json algorithm"),
        }

        info!("Imported Ed25519 private key to {}", output_path.display());
    }
}
