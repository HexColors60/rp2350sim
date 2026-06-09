//! Rhai bindings.
#![allow(dead_code)]

use rhai::{Engine, Module};

/// Register all bindings with the engine.
pub fn register_bindings(engine: &mut Engine) {
    // Register simulator API
    let mut simulator_module = Module::new();
    simulator_module.set_native_fn("reset", |_: &mut ()| Ok(()));
    simulator_module.set_native_fn("step", |_: &mut ()| Ok(()));
    simulator_module.set_native_fn("run", |_: &mut ()| Ok(()));
    simulator_module.set_native_fn("stop", |_: &mut ()| Ok(()));
    engine.register_global_module(simulator_module.into());

    // Register memory API
    let mut memory_module = Module::new();
    memory_module.set_native_fn("read_byte", |_: &mut (), _addr: i64| Ok(0i64));
    memory_module.set_native_fn("write_byte", |_: &mut (), _addr: i64, _value: i64| Ok(()));
    memory_module.set_native_fn("read_word", |_: &mut (), _addr: i64| Ok(0i64));
    memory_module.set_native_fn("write_word", |_: &mut (), _addr: i64, _value: i64| Ok(()));
    engine.register_global_module(memory_module.into());

    // Register GPIO API
    let mut gpio_module = Module::new();
    gpio_module.set_native_fn("set_pin", |_: &mut (), _pin: i64, _value: bool| Ok(()));
    gpio_module.set_native_fn("get_pin", |_: &mut (), _pin: i64| Ok(false));
    gpio_module.set_native_fn("set_direction", |_: &mut (), _pin: i64, _output: bool| Ok(()));
    engine.register_global_module(gpio_module.into());
}