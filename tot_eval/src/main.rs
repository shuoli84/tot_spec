use anyhow::{anyhow, bail};
use rustyline::error::ReadlineError;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, Editor, EventHandler, KeyCode, KeyEvent, Modifiers};
use rustyline::{Completer, Helper, Highlighter, Hinter, Validator};

use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tot_lang::runtime::Vm;
use tot_lang::type_repository::TypeRepository;
use tot_lang::{Value, VmBehavior};
use tot_spec::codegen::context::Context;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        long,
        help = "root folder for all spec, will scan all specs in the folder recursively"
    )]
    spec_folder: PathBuf,
}

#[derive(Completer, Helper, Highlighter, Hinter, Validator)]
struct InputValidator {
    #[rustyline(Validator)]
    brackets: MatchingBracketValidator,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let spec_folder = args.spec_folder;
    let mut vm = new_vm(spec_folder)?;

    let h = InputValidator {
        brackets: MatchingBracketValidator::new(),
    };
    let mut rl = Editor::new()?;
    rl.set_helper(Some(h));
    rl.bind_sequence(
        KeyEvent(KeyCode::Char('s'), Modifiers::CTRL),
        EventHandler::Simple(Cmd::Newline),
    );

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                match vm.eval(line.as_str()).await {
                    Ok(_) => {
                        continue;
                    }
                    Err(err) => {
                        println!("met error: {err}");
                        continue;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

fn new_vm(spec_folder: PathBuf) -> anyhow::Result<Vm> {
    let context = Context::new_from_folder(&spec_folder)?;
    let type_repository = Arc::new(TypeRepository::new(context));
    Ok(Vm::new(
        Box::new(RuntimeBehavior::new(type_repository.clone())),
        type_repository,
    ))
}

#[derive(Debug)]
struct RuntimeBehavior {
    type_repository: Arc<TypeRepository>,
}

impl RuntimeBehavior {
    pub fn new(type_repository: Arc<TypeRepository>) -> Self {
        Self { type_repository }
    }
}

#[async_trait::async_trait]
impl VmBehavior for RuntimeBehavior {
    async fn runtime_call_method(
        &mut self,
        method: &str,
        params: &[Value],
    ) -> anyhow::Result<Value> {
        match method {
            "json" => {
                let param = &params[0];
                let str_val = param.as_str().unwrap();
                return Ok(serde_json::from_str::<Value>(str_val)?);
            }
            "_type_info" => {
                if params.is_empty() {
                    println!("try: type_info(\"i32\") or type_info(\"type::path\")");
                } else {
                    let param = &params[0];
                    let type_path = param
                        .as_str()
                        .ok_or_else(|| anyhow!("only string supported"))?;
                    let model_or_type = self.type_repository.type_for_path(type_path)?;
                    dbg!(model_or_type);
                }

                Ok(Value::Null)
            }
            "_specs" => {
                let context = self.type_repository.context();
                for (spec_id, _def) in context.iter_specs() {
                    let spec_path = context.path_for_spec(spec_id).unwrap();
                    println!("{spec_id:?}: {spec_path:?}");
                }
                Ok(Value::Null)
            }
            "_types" => {
                let context = self.type_repository.context();
                for (spec_id, def) in context.iter_specs() {
                    println!("spec: {spec_id:?}");
                    for model in &def.models {
                        println!("  {}", model.name);
                    }
                }
                Ok(Value::Null)
            }
            "_methods" => {
                let context = self.type_repository.context();
                for (spec_id, def) in context.iter_specs() {
                    println!("spec: {spec_id:?}");
                    for method in &def.methods {
                        println!(
                            "  {}({:?}) -> {:?}",
                            method.name, method.request, method.response
                        );
                    }
                }
                Ok(Value::Null)
            }
            _ => bail!("method [{method}] not supported"),
        }
    }
}
