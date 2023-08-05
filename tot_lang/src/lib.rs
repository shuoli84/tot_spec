//! tot_lang is a naive programming lang that aims to provide
//! both dynamic execution and static codegen(to rust). It utilize
//! tot_spec's type definition to define types.
//!
//! ## why
//! In our use case, models and methods are defined and implemented
//! first, they are the building blocks.
//! Then some users build biz logic on top of the methods.
//! tot_lang tries to provide a new abstraction layer to enable the
//! try_and_build usage pattern.
//! Try: easy to develop, iterate and visualize.
//! Build: correct, fast, suitable for production long running.
//!
//! ## How
//! tot_lang can be used as both script and codegen, and we try to design
//! and implement the lang so the two approaches have exact same output.
//!
//! ```tot
//! // assign "hello" string value to local variable hello
//! let hello: string = "hello";
//!
//! {
//!   // start a new scope
//!
//!   // create a new variable hello_world
//!   let hello_world: string = hello + " world";
//!   // rebind the hello name
//!   let hello: integer = 123;
//! }
//!
//! // foo_bar is the model def from tot_spec, this assign
//! // has same output as from_json
//! let request: foo_bar::Request = {
//!   "foo": "bar"
//! };
//!
//! // create a copy from foo (currently the lang doesn't support reference)
//! let foo = request.foo;
//!
//! // call foo_bar's process method
//! let response = foo_bar::process(request);
//! // convert the response to process_2's request, it is just like
//! // object -> json -> object conversion.
//! let response = foo_bar::process_2(response);
//!
//! // also call user defined functions
//! send_to_kafka("process_2_response", response);
//!
//! ```

use std::fmt;

pub mod ast;
pub mod codegen;
pub mod program;
pub mod runtime;

pub use serde_json::Value;

/// We need the user to provide A behavior to inject customized logic
#[async_trait::async_trait]
pub trait Behavior: fmt::Debug {
    /// Execute an method with name
    async fn runtime_call_method(
        &mut self,
        method: &str,
        params: &[Value],
    ) -> anyhow::Result<Value>;

    /// the return type for method
    /// consider return func signature in future?
    fn return_type_for_method(&mut self, name: &str) -> anyhow::Result<String>;

    fn codegen_for_func_signature(
        &mut self,
        name: &str,
        params: &[String],
        ret_type: Option<&str>,
    ) -> anyhow::Result<String> {
        let params = params.join(",");
        Ok(if let Some(ret_type) = ret_type {
            format!("fn {name}({params}) -> {ret_type}")
        } else {
            format!("fn {name}({params}")
        })
    }

    /// gen type for path
    /// e.g: print => println!
    fn codegen_for_type(&mut self, path: &str) -> anyhow::Result<String> {
        if path.eq("debug") {
            return Ok("dbg!".to_string());
        }

        Ok(path.to_string())
    }

    /// gen for call
    fn codegen_for_call(&mut self, path: &str, params: &[String]) -> anyhow::Result<String> {
        let params_code = params.join(", ");
        Ok(format!("{path}({params_code})"))
    }

    /// customization point for return expression
    /// is_last: whether this express is the body's last value expr, if false, then it is in body
    ///       return
    fn codegen_for_return(&mut self, expr: &str, is_last: bool) -> anyhow::Result<String> {
        if is_last {
            Ok(format!("{expr}"))
        } else {
            Ok(format!("return {expr};"))
        }
    }
}
