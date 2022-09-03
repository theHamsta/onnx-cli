pub mod onnx {
    include!(concat!(env!("OUT_DIR"), "/onnx.rs"));
}

use std::{
    collections::HashSet,
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
    #[clap(long, short, value_parser)]
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
    println!("doc_string {:?}:", model.doc_string);
    println!("ir_version {:?}:", model.ir_version);
    if let Some(graph) = model.graph {
        println!("graph.doc_string {:?}:", model.doc_string);
        for (i, node) in graph.node.iter().enumerate() {
            println!("node {i}: {:?}:", node.name);
        }
        let initialized: HashSet<String> = graph
            .initializer
            .iter()
            .cloned()
            .map(|i| i.name)
            //.chain(graph.sparse_initializer.iter().map(|i| i.name))
            .collect();

        for (i, input) in graph.input.iter().enumerate() {
            if !initialized.contains(&input.name) {
                println!("input {i}: {:?} {:?}:", input.name, input.r#type);
            }
        }
        for (i, ouput) in graph.output.iter().enumerate() {
            println!("output {i}: {:?}:", ouput.name);
        }
    }
    Ok(())
}
