use clap::Parser;
use tot_spec::Definition;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    spec: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();

    let spec_content = std::fs::read_to_string(args.spec).unwrap();
    let def = serde_yaml::from_str::<Definition>(&spec_content).unwrap();

    println!("{:#?}", def);
}
