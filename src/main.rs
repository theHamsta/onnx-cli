#![feature(drain_filter)]

pub mod onnx {
    include!(concat!(env!("OUT_DIR"), "/onnx.rs"));
}

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use clap::Parser;
use prost::Message;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: Action,
    /// ONNX file to process
    #[clap(value_parser)]
    onnx: PathBuf,
}

fn open_file(path: &Path) -> anyhow::Result<onnx::ModelProto> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    Ok(onnx::ModelProto::decode(&buffer[..])?)
}

fn save_file(path: &Path, model: &onnx::ModelProto) -> anyhow::Result<()> {
    let f = File::create(path)?;
    let mut writer = BufWriter::new(f);
    let buffer = model.encode_to_vec();
    let _ = writer.write(&buffer)?;
    Ok(())
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    Info,
    MakeDynamic,
    Remove {
        #[clap(value_parser)]
        output: PathBuf,
        #[clap(value_parser)]
        ops: Vec<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    pretty_env_logger::init();

    let mut model =
        open_file(&args.onnx).with_context(|| format!("Failed to open {:?}", args.onnx))?;

    match args.action {
        Action::Info => print_info(&model)?,
        Action::MakeDynamic => make_dynamic(&mut model)?,
        Action::Remove { output, ops } => {
            remove_ops(&mut model, ops)?;
            save_file(&output, &model).with_context(|| format!("Failed to save to {output:?}"))?;
        }
    }
    Ok(())
}

fn remove_ops(model: &mut onnx::ModelProto, ops: Vec<String>) -> anyhow::Result<()> {
    let nodes = &mut model
        .graph
        .as_mut()
        .ok_or_else(|| anyhow!("ONNX has not Graph"))?
        .node;
    let layer_lookup: HashMap<_, _> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.name.clone(), i))
        .collect();
    let mut new_connections = Vec::new();

    for node in nodes.iter() {
        if ops.contains(&node.name) {
            if let ([input], [output]) = (&node.input[..], &node.output[..]) {
                new_connections.push((layer_lookup[input], layer_lookup[output]))
            }
        }
    }

    for (i, o) in new_connections {
        let name = nodes[o].name.to_string();
        nodes[i].output.push(name);
        let name = nodes[i].name.to_string();
        nodes[o].input.push(name);
    }

    nodes.drain_filter(|node| {
        node.input.drain_filter(|n| !ops.contains(n));
        node.output.drain_filter(|n| !ops.contains(n));
        !ops.contains(&node.name)
    });
    Ok(())
}

fn make_dynamic(model: &mut onnx::ModelProto) -> anyhow::Result<()> {
    let graph = model
        .graph
        .as_mut()
        .ok_or_else(|| anyhow!("No graph in ONNX"))?;
    let initialized: HashSet<String> = graph
        .initializer
        .iter()
        .cloned()
        .map(|i| i.name)
        //.chain(graph.sparse_initializer.iter().map(|i| i.name))
        .collect();

    for (i, input) in graph.input.iter().enumerate() {
        if !initialized.contains(&input.name) {
            if let Some(size) = &input.r#type {
                println!("input {i}: {:?} {:?}:", input.name, size);
            }
        }
    }

    Ok(())
}

fn print_info(model: &onnx::ModelProto) -> anyhow::Result<()> {
    println!("doc_string {:?}:", model.doc_string);
    println!("ir_version {:?}:", model.ir_version);
    if let Some(graph) = &model.graph {
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
