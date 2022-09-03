pub mod onnx {
    include!(concat!(env!("OUT_DIR"), "/onnx.rs"));
}

fn main() {
    println!("Hello, world!");
}
