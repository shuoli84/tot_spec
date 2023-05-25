use clap::Parser;
use tot_spec::{codegen, Context, Definition};

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

    let spec_content = std::fs::read_to_string(&args.spec).unwrap();
    let def = serde_yaml::from_str::<Definition>(&spec_content).unwrap();

    println!("{} models loaded", def.models.len());

    let output = if args.codegen.eq("rs_serde") {
        codegen::rs_serde::render(&def).unwrap()
    } else if args.codegen.eq("py_dataclass") {
        codegen::py_dataclass::render(&def).unwrap()
    } else if args.codegen.eq("swift_codable") {
        codegen::swift_codable::render(&def).unwrap()
    } else if args.codegen.eq("java_jackson") {
        let spec_content = std::fs::read_to_string(&args.spec).unwrap();
        let def = serde_yaml::from_str::<Definition>(&spec_content).unwrap();

        println!("{} models loaded", def.models.len());

        let context = Context::load_from_path(&args.spec).unwrap();
        codegen::java_jackson::render(&def, &context, args.output.as_ref().unwrap()).unwrap();
        return;
    } else {
        unimplemented!()
    };

    if let Some(output_file) = args.output {
        std::fs::write(&output_file, output).unwrap();
        println!("write output to {:?}", output_file);
    }
}
