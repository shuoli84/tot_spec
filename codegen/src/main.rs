use clap::Parser;
use tot_spec::codegen::swagger::Swagger;
use tot_spec::codegen::{
    java_jackson::JavaJackson, py_dataclass::PyDataclass, rs_serde::RsSerde,
    swift_codable::SwiftCodable, Codegen,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "root folder for all spec, will scan all specs in the folder recursivly"
    )]
    input: std::path::PathBuf,

    #[arg(short, long, default_value = "rs_serde")]
    codegen: String,

    #[arg(
        short,
        long,
        default_value = "examples/spec/",
        help = "output path, if input is folder, then output must be folder"
    )]
    output: std::path::PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let codegen: Box<dyn Codegen> = match args.codegen.as_str() {
        "rs_serde" => Box::new(RsSerde::default()),
        "java_jackson" => Box::new(JavaJackson::default()),
        "swift_codable" => Box::new(SwiftCodable::default()),
        "py_dataclass" => Box::new(PyDataclass::default()),
        "swagger" => Box::new(Swagger::default()),
        _ => anyhow::bail!("unknown codegen name"),
    };

    let output = std::fs::canonicalize(args.output).unwrap();

    codegen.generate_for_folder(&args.input, &output)?;
    Ok(())
}
