fn main() {
    println!("cargo:rerun-if-changed=../../proto/soma.proto");
    prost_build::compile_protos(&["../../proto/soma.proto"], &["../../proto"])
        .expect("compile Soma protocol");
}
