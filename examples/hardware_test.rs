#![no_std]

/// Hardware regression tests.
/// Need P0.03 and P0.04 to be connected (on a nrf52-dk).
extern crate alloc;

use alloc::string::String;
use core::fmt::Write;
use futures::future;
use libtock::console::Console;
use libtock::gpio::{GpioPinUnitialized, InputMode};
use libtock::result::TockResult;
use libtock::timer;
use libtock::timer::Duration;
use libtock::Hardware;

static mut STATIC: usize = 0;

trait MyTrait {
    fn do_something_with_a_console(&self, console: &mut Console);
}

impl MyTrait for usize {
    fn do_something_with_a_console(&self, console: &mut Console) {
        writeln!(console, "trait_obj_value_usize = {}", &self).unwrap();
    }
}

impl MyTrait for String {
    fn do_something_with_a_console(&self, console: &mut Console) {
        writeln!(console, "trait_obj_value_string = {}", &self).unwrap();
    }
}

#[libtock::main]
async fn main() -> TockResult<()> {
    let Hardware { console_driver, .. } = libtock::retrieve_hardware()?;
    let mut console = console_driver.create_console();
    writeln!(console, "[test-results]")?;

    test_heap(&mut console);
    test_formatting(&mut console);
    test_static_mut(&mut console);

    test_gpio(&mut console);

    test_trait_objects(&mut console)?;

    test_callbacks_and_wait_forever(&mut console).await
}

fn test_heap(console: &mut Console) {
    let mut string = String::from("heap_test = \"Heap ");
    string.push_str("works.\"");
    writeln!(console, "{}", string).unwrap();
}

fn test_formatting(console: &mut Console) {
    writeln!(console, "formatting =  {}", String::from("works")).unwrap();
}

/// needs P0.03 and P0.04 to be connected
/// Output order should be:
/// trait_obj_value_usize = 1
/// trait_obj_value_string = string
fn test_trait_objects(console: &mut Console) -> TockResult<()> {
    let pin_in = GpioPinUnitialized::new(0);
    let pin_in = pin_in.open_for_read(None, InputMode::PullDown)?;

    let pin_out = GpioPinUnitialized::new(1);
    let pin_out = pin_out.open_for_write()?;
    pin_out.set_high()?;

    let string = String::from("string");

    let x = if pin_in.read() {
        &1usize as &dyn MyTrait
    } else {
        &string as &dyn MyTrait
    };

    let y = if !pin_in.read() {
        &1usize as &dyn MyTrait
    } else {
        &string as &dyn MyTrait
    };

    x.do_something_with_a_console(console);
    y.do_something_with_a_console(console);
    Ok(())
}

fn test_static_mut(console: &mut Console) {
    increment_static_mut();

    writeln!(console, "should_be_one = {}", unsafe { STATIC }).unwrap();
}

/// needs P0.03 and P0.04 to be connected
fn test_gpio(console: &mut Console) {
    let pin_in = GpioPinUnitialized::new(0);
    let pin_in = pin_in
        .open_for_read(None, InputMode::PullDown)
        .ok()
        .unwrap();

    let pin_out = GpioPinUnitialized::new(1);
    let pin_out = pin_out.open_for_write().ok().unwrap();
    pin_out.set_high().ok().unwrap();

    writeln!(console, "gpio_works = {}", pin_in.read()).unwrap();
}

async fn test_callbacks_and_wait_forever(console: &mut Console) -> TockResult<()> {
    let mut with_callback = timer::with_callback(|_, _| {
        writeln!(console, "callbacks_work = true").unwrap();
        writeln!(console, "all_tests_run = true").unwrap();
    });

    let mut timer = with_callback.init()?;

    timer.set_alarm(Duration::from_ms(500))?;

    future::pending().await
}

#[inline(never)]
fn increment_static_mut() {
    unsafe { STATIC += 1 };
}
