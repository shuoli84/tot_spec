use clap::Parser;
use tot_spec::{codegen, Definition};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "examples/spec/spec.yaml")]
    spec: std::path::PathBuf,

    #[arg(short, long, default_value = "rs_serde")]
    codegen: String,

    #[arg(short, long, default_value = "examples/spec/example_spec.rs")]
    output: Option<std::path::PathBuf>,
}

fn main() {
    let args = Args::parse();

    let spec_content = std::fs::read_to_string(args.spec).unwrap();
    let def = serde_yaml::from_str::<Definition>(&spec_content).unwrap();

    println!("{:#?}", def);

    let output = if args.codegen.eq("rs_serde") {
        codegen::rs_serde::render(&def).unwrap()
    } else if args.codegen.eq("py_dataclass") {
        codegen::py_dataclass::render(&def).unwrap()
    } else {
        unimplemented!()
    };

    if let Some(output_file) = args.output {
        std::fs::write(&output_file, output).unwrap();
        println!("write output to {:?}", output_file);
    }
}
