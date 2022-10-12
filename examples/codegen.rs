use clap::Parser;
use tot_spec::{codegen, Definition};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "examples/spec/spec.yaml")]
    spec: std::path::PathBuf,

    #[arg(short, long, default_value = "examples/spec/example_spec.rs")]
    output: Option<std::path::PathBuf>,
}

fn main() {
    let args = Args::parse();

    let spec_content = std::fs::read_to_string(args.spec).unwrap();
    let def = serde_yaml::from_str::<Definition>(&spec_content).unwrap();

    println!("{:#?}", def);

    if let Some(output_file) = args.output {
        let output = codegen::rs_serde::render(&def).unwrap();
        std::fs::write(&output_file, output).unwrap();
        println!("write output to {:?}", output_file);
    }
}
