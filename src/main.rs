pub mod onnx {
    include!(concat!(env!("OUT_DIR"), "/onnx.rs"));
}

use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use clap::Parser;
use prost::Message;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: Action,
    /// ONNX file to process
    onnx: PathBuf,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    Info,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let f = File::open(args.onnx)?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let model = onnx::ModelProto::decode(&buffer[..])?;
    match args.action {
        Action::Info => print_info(model)?,
    }
    Ok(())
}

fn print_info(model: onnx::ModelProto) -> anyhow::Result<()> {

    println!("ir_version {:?}:", model.ir_version);
    Ok(())
}
