use clap::Parser;
use tot_spec::{
    codegen::{self, spec_folder::SpecFolder},
    Context, Definition,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    spec: Option<std::path::PathBuf>,

    #[arg(
        long,
        help = "root folder for all spec, will scan all specs in the folder recursivly"
    )]
    spec_folder: Option<std::path::PathBuf>,

    #[arg(short, long, default_value = "rs_serde")]
    codegen: String,

    #[arg(
        short,
        long,
        default_value = "examples/spec/example_spec.rs",
        help = "output path, if input is folder, then output must be folder"
    )]
    output: std::path::PathBuf,
}

impl Args {
    fn get_input(&self) -> SpecInput {
        match (&self.spec, &self.spec_folder) {
            (None, None) => {
                panic!("spec or spec_folder must be specified");
            }
            (Some(path), None) => SpecInput::File(path.clone()),
            (None, Some(path)) => SpecInput::Folder(path.clone()),
            _ => {
                panic!("spec and spec_folder can not be specified at the same time");
            }
        }
    }
}

enum SpecInput {
    File(std::path::PathBuf),
    Folder(std::path::PathBuf),
}

fn main() {
    let args = Args::parse();

    match args.get_input() {
        SpecInput::File(ref spec) => generate_one_spec(spec, &args.codegen, &args.output),
        SpecInput::Folder(folder) => generate_for_folder(&folder, &args.codegen, &args.output),
    }
}

fn generate_for_folder(folder: &std::path::PathBuf, codegen: &str, output: &std::path::PathBuf) {
    use walkdir::WalkDir;

    std::fs::create_dir_all(output).unwrap();
    let mut spec_folder = SpecFolder::new();

    for entry in WalkDir::new(folder) {
        let entry = entry.unwrap();
        let entry_path = entry.path();

        if entry_path.is_dir() {
            // move logic to spec stack handling
            if codegen == "py_dataclass" {
                // python dataclass codegen needs to generate __init__.py for each folder
                let relative_path = entry_path.strip_prefix(folder).unwrap();
                let output_folder = output.join(relative_path);
                std::fs::create_dir_all(output_folder).unwrap();
                let init_file = output.join(relative_path).join("__init__.py");
                std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(init_file)
                    .unwrap();
            }
            continue;
        }
        if !entry_path.is_file() {
            continue;
        }
        if !entry_path
            .extension()
            .map(|ext| ext == "yaml")
            .unwrap_or_default()
        {
            continue;
        }

        let relative_path = entry_path.strip_prefix(folder).unwrap();
        spec_folder.insert(relative_path);

        // now we get a file ends with yaml, build the output path
        // todo: how to map spec to output path is also codegen dependant, maybe move into core?
        let output = match codegen {
            "java_jackson" => {
                // java jackson may generate multiple java files, it is controlled by codegen, so
                // we just pass the output folder
                output.clone()
            }
            "rs_serde" => {
                // todo: rust also needs to generate mod.rs
                let mut output = output.clone();
                output.push(relative_path);
                output.set_extension("rs");
                output
            }
            "py_dataclass" => {
                let mut output = output.clone();
                output.push(relative_path);
                output.set_extension("py");
                output
            }
            "swift_codable" => {
                let mut output = output.clone();
                output.push(relative_path);
                output.set_extension("swift");
                output
            }
            _ => {
                panic!("unknown codegen {}", codegen);
            }
        };

        generate_one_spec(&entry_path, codegen, &output)
    }

    spec_folder.foreach_entry_recursively(|entry| {
        if codegen == "rs_serde" {
            let outputs = codegen::rs_serde::render_folder(entry).unwrap();
            for (file_relative_path, content) in outputs {
                let file_path = output.join(file_relative_path);
                println!("write output to {:?}", file_path);
                std::fs::write(file_path, content).unwrap();
            }
        }
    });
}

fn generate_one_spec(spec: &std::path::Path, codegen: &str, output: &std::path::PathBuf) {
    println!("generating codegen={codegen} spec={spec:?} output={output:?}");
    let spec_content = std::fs::read_to_string(spec).unwrap();
    let def = serde_yaml::from_str::<Definition>(&spec_content).unwrap();

    let parent_folder = output.parent().unwrap();
    std::fs::create_dir_all(parent_folder).unwrap();

    let code = match codegen {
        "rs_serde" => {
            let context = Context::load_from_path(spec).unwrap();
            codegen::rs_serde::render(&def, &context).unwrap()
        }
        "py_dataclass" => {
            let context = Context::load_from_path(spec).unwrap();
            codegen::py_dataclass::render(&def, &context).unwrap()
        }
        "swift_codable" => codegen::swift_codable::render(&def).unwrap(),
        "java_jackson" => {
            let spec_content = std::fs::read_to_string(spec).unwrap();
            let def = serde_yaml::from_str::<Definition>(&spec_content).unwrap();

            let context = Context::load_from_path(spec).unwrap();
            codegen::java_jackson::render(&def, &context, output).unwrap();
            return;
        }
        _ => {
            panic!("unknown codegen {}", codegen);
        }
    };

    std::fs::write(&output, code).unwrap();
    println!("write output to {:?}", output);
}
