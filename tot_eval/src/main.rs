use anyhow::bail;
use rustyline::error::ReadlineError;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, Editor, EventHandler, KeyCode, KeyEvent, Modifiers};
use rustyline::{Completer, Helper, Highlighter, Hinter, Validator};

use std::path::PathBuf;
use std::sync::Arc;
use tot_lang::runtime::Vm;
use tot_lang::type_repository::TypeRepository;
use tot_lang::{Value, VmBehavior};
use tot_spec::codegen::context::Context;

#[derive(Completer, Helper, Highlighter, Hinter, Validator)]
struct InputValidator {
    #[rustyline(Validator)]
    brackets: MatchingBracketValidator,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let h = InputValidator {
        brackets: MatchingBracketValidator::new(),
    };
    let mut rl = Editor::new()?;
    rl.set_helper(Some(h));
    rl.bind_sequence(
        KeyEvent(KeyCode::Char('s'), Modifiers::CTRL),
        EventHandler::Simple(Cmd::Newline),
    );

    let mut vm = new_vm();

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

fn new_vm() -> Vm {
    let context =
        Context::new_from_folder(&PathBuf::from("tot_lang/src/codegen/fixtures")).unwrap();
    let type_repository = Arc::new(TypeRepository::new(context));
    Vm::new(Box::new(RuntimeBehavior::default()), type_repository)
}

#[derive(Debug, Default)]
struct RuntimeBehavior {}

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
            "foo" => return Ok(Value::String("foo".into())),
            "bar" => return Ok(Value::String("bar".into())),
            _ => bail!("{method} not supported"),
        }
    }
}
