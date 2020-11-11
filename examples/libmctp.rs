#![no_std]
/// This is a example of a MCTP master device
use libmctp::smbus::{MCTPSMBusContext, VendorIDFormat};
use libtock::i2c_master::I2cBuffer;
use libtock::result::TockResult;
use libtock::syscalls;
use libtock::{print, println};

libtock_core::stack_size! {0x800}

// The address of this device
const MY_ID: u8 = 0x23;
const DEST_ID: u8 = 0x10;
// Support vendor defined protocol 0x7E
const MSG_TYPES: [u8; 1] = [0x7E];
// Specify a PCI vendor ID that we support
const VENDOR_IDS: [libmctp::smbus::VendorIDFormat; 1] = [VendorIDFormat {
    // PCI Vendor ID
    format: 0x00,
    // PCI VID
    data: 0x1414,
    // Extra data
    numeric_value: 4,
}];

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;
    drivers.console.create_console();
    println!("Starting libmctp example");
    let i2c_driver = drivers.i2c.init_driver()?;

    // Check that I2C exists
    let present = i2c_driver.check_present();

    match present {
        Ok(_) => {
            println!("Found the I2C device");
        }
        Err(_) => {
            println!("No I2C device, yielding");
            loop {
                unsafe { syscalls::raw::yieldk() };
            }
        }
    }

    println!("Setting callback");
    let mut callback = |_, _| {
        println!("I2C Callback");
    };

    let _subscription = i2c_driver.subscribe(&mut callback)?;
    println!("Creating MCTP SMBus Context");
    let ctx = MCTPSMBusContext::new(MY_ID, &MSG_TYPES, &VENDOR_IDS);

    let mut buf: [u8; 32] = [0; 32];

    println!("Creating the request");
    let len = ctx
        .get_request()
        .get_vendor_defined_message_support(0xB, 0, &mut buf);

    println!("Creating master write buffer");
    let mut master_write_buffer = I2cBuffer::default();
    // Skip the first byte, as that is the destination address
    for (i, d) in buf[1..].iter().enumerate() {
        master_write_buffer[i] = *d;
    }
    let dest_buffer = i2c_driver.init_buffer(&mut master_write_buffer)?;
    println!("  done");

    let _ = i2c_driver.write(DEST_ID as usize, len.unwrap());

    unsafe { syscalls::raw::yieldk() };

    // Cause a delay before the I2C read
    for _i in 0..10000 {
        print!(".");
    }

    let _ = i2c_driver.read(DEST_ID as usize, 18);

    unsafe { syscalls::raw::yieldk() };

    // Copy into a temp buffer, with space for the destination address
    let mut temp_buffer = [0; libtock::hmac::DEST_BUFFER_SIZE];
    dest_buffer.read_bytes(&mut temp_buffer[1..]);

    // Set the destination address as this isn't filled in the buffer
    temp_buffer[0] = DEST_ID << 1;

    // HACK: Set the PEC as Cerebus doesn't
    temp_buffer[18] = 0x5e;

    // Print the buffer
    for buf in temp_buffer[0..19]
        .iter()
        .take(libtock::hmac::DEST_BUFFER_SIZE)
    {
        println!("{:#x}", *buf);
    }

    for _i in 0..100 {
        print!(".");
    }

    // Decode the response
    let ret = ctx.decode_packet(&temp_buffer[0..19]);

    // Print the outcome of the decode
    println!("ret: {:?}", ret);

    loop {
        unsafe { syscalls::raw::yieldk() };
    }
}
