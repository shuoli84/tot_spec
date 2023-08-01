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

pub mod ast;
pub mod codegen;
pub mod program;
pub mod runtime;

/// We need the user to provide A behavior to inject customized logic
#[async_trait::async_trait]
pub trait Behavior {}
