#![no_std]
/// This is a basic KV Store example
use libtock::kv_store::{KVStoreKeyBuffer, KVStoreValueBuffer};
use libtock::result::TockResult;
use libtock::syscalls;
use libtock::{print, println};

libtock_core::stack_size! {0x800}

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;
    drivers.console.create_console();
    println!("Starting KV Store");
    let kv_store_driver = drivers.kv_store.init_driver()?;

    println!("Creating key buffer");
    let mut key_buffer = KVStoreKeyBuffer::default();
    let key: &[u8; 16] = b"tickfs-super-key";

    for (i, d) in key.iter().enumerate() {
        key_buffer[i] = *d;
    }

    // println!("Userspace key: {:#x?}", key_buffer);

    let _key_buffer = kv_store_driver.init_key_buffer(&mut key_buffer)?;
    println!("  done");

    println!("Creating value buffer");
    let mut value_buffer = KVStoreValueBuffer::default();
    let value_buffer = kv_store_driver.init_value_buffer(&mut value_buffer)?;
    println!("  done");

    let mut temp_buffer = [0; libtock::kv_store::VALUE_BUFFER_SIZE];

    println!("Setting callback and running");
    let mut callback = |_, _| {
        println!("Operation Complete, printing data");
        value_buffer.read_bytes(&mut temp_buffer[..]);

        for buf in temp_buffer.iter().take(libtock::kv_store::VALUE_BUFFER_SIZE) {
            print!("{:#x?}", *buf);
        }
        println!();
    };

    let _subscription = kv_store_driver.subscribe(&mut callback)?;
    kv_store_driver.get_key(key.len())?;

    loop {
        unsafe { syscalls::raw::yieldk() };
    }
}
