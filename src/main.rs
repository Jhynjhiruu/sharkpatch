use std::fs::{read, write};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use clap_num::maybe_hex;
use clap::Parser;
use hex::FromHexError;
use thiserror::Error;
use unfmt::unformat;

#[derive(Debug, Clone)]
struct Patch {
    offset: usize,
    bytes: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum ParsePatchError {
    #[error("no separator in patch string")]
    MissingSeparator,

    #[error("unable to split patch string")]
    UnformatError,

    #[error(transparent)]
    HexDecodeError(#[from] FromHexError),

    #[error(transparent)]
    IntError(#[from] ParseIntError),
}

impl std::fmt::Display for Patch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04X}:{}", self.offset, hex::encode_upper(&self.bytes))
    }
}

impl FromStr for Patch {
    type Err = ParsePatchError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.contains(':') {
            return Err(ParsePatchError::MissingSeparator);
        }

        let Some((offset, bytes)) = unformat!("{}:{}", s) else {
            return Err(ParsePatchError::UnformatError);
        };

        let offset = usize::from_str_radix(offset, 16)?;
        let bytes = hex::decode(bytes)?;

        Ok(Self { offset, bytes })
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file
    infile: PathBuf,

    /// Output file
    outfile: PathBuf,

    /// Patches to apply
    #[arg(short, long)]
    patches: Vec<Patch>,

    /// Size to truncate to
    #[arg(short, long, value_parser = maybe_hex::<usize>)]
    truncate_to: Option<usize>
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut data = read(args.infile)?;

    for patch in args.patches {
        let len = patch.bytes.len();
        data[patch.offset..patch.offset + len].copy_from_slice(&patch.bytes);
    }

    let data = if let Some(t) = args.truncate_to {
        &data[0..t.min(data.len())]
    } else {
        &data
    };

    write(args.outfile, data)?;

    Ok(())
}
