//! This is a featured CTAP example
//! WARNING! This currently uses unsound crypto operations
//! This is only a demo and should not be used in real enviroments
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::vec::Vec;
use core::alloc::Layout;
use core::time::Duration;
use libmctp::smbus::MCTPSMBusContext;
use libmctp::vendor_packets::VendorIDFormat;
use libtock::i2c_master::{I2cBuffer, I2cDriverFactory};
use libtock::{print, println};
use libtock::result::TockResult;
use libtock::syscalls;
use manticore::crypto::rsa;
use manticore::hardware;
use manticore::io::*;
use manticore::mem::BumpArena;
use manticore::net::{self, HostPort, HostRequest, HostResponse};
use manticore::protocol::capabilities;
use manticore::protocol::*;
use manticore::server::pa_rot;

libtock_core::stack_size! {0x800}

// The address of this device
const MY_ID: u8 = 0x23;
const DEST_ID: u8 = 0x10;
// Support vendor defined protocol 0x7E
const MSG_TYPES: [u8; 1] = [0x7E];
// Specify a PCI vendor ID that we support
const VENDOR_IDS: [VendorIDFormat; 1] = [VendorIDFormat {
    // PCI Vendor ID
    format: 0x00,
    // PCI VID
    data: 0x1414,
    // Extra data
    numeric_value: 4,
}];

struct Identity {
    firmware_version: [u8; 32],
    unique_id: Vec<u8>,
}

impl Identity {
    /// Creates a new `fake::Identity`.
    pub fn new(firmware_version: &[u8], unique_id: &[u8]) -> Self {
        let mut buf: [u8; 32] = [0; 32];
        buf[0..firmware_version.len()].copy_from_slice(firmware_version);

        Self {
            firmware_version: buf,
            unique_id: unique_id.to_vec(),
        }
    }
}

impl hardware::Identity for Identity {
    fn firmware_version(&self) -> &[u8; 32] {
        &self.firmware_version
    }

    fn unique_device_identity(&self) -> &[u8] {
        &self.unique_id[..]
    }
}

struct Reset {
    resets: u32,
    uptime: Duration,
}

impl Reset {
    pub fn new(resets: u32, uptime: Duration) -> Self {
        Self { resets, uptime }
    }
}

impl hardware::Reset for Reset {
    fn resets_since_power_on(&self) -> u32 {
        println!("Reset: resets_since_power_on");

        0
    }

    fn uptime(&self) -> Duration {
        println!("Reset: uptime");
        Duration::new(5, 0)
    }
}

struct Error {}

struct PublicKey {}

impl PublicKey {
    pub fn new() -> Self {
        Self {}
    }
}

impl rsa::PublicKey for PublicKey {
    fn len(&self) -> rsa::ModulusLength {
        println!("Reset: len");

        rsa::ModulusLength::Bits2048
    }
}

struct Builder {}

impl Builder {
    pub fn new() -> Self {
        Self {}
    }
}

impl rsa::Builder for Builder {
    type Engine = Engine;

    fn supports_modulus(&self, len: rsa::ModulusLength) -> bool {
        println!("Reset: supports_modulus");

        true
    }

    fn new_engine(&self, key: PublicKey) -> Result<Engine, Error> {
        println!("Reset: new_engine");

        Ok(Engine::new())
    }
}

struct Engine {}

impl Engine {
    pub fn new() -> Self {
        Self {}
    }
}

impl rsa::Engine for Engine {
    type Error = Error;
    type Key = PublicKey;

    fn verify_signature(&mut self, signature: &[u8], message: &[u8]) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

pub struct ManticoreHost<'a> {
    ctx: MCTPSMBusContext<'a>,
    i2c: I2cDriverFactory,
}

impl<'a> ManticoreHost<'a> {
    pub fn new(ctx: MCTPSMBusContext<'a>, i2c: I2cDriverFactory) -> Self {
        Self { ctx, i2c }
    }

    /// Schedules a new request to be received, with the given request parts.
    ///
    /// Calling this function will make `receive()` start working; otherwise,
    /// it will assert that the port is disconnected.
    pub fn request(&mut self, header: Header, message: &[u8]) {
        println!("ManticoreHost: request");

        let i2c_driver;
        match self.i2c.init_driver() {
            Err(_) => {
                panic!("I2C init error");
            }
            Ok(driver) => {
                i2c_driver = driver;
            }
        }

        println!("Setting callback");
        let mut callback = |_, _| {
            println!("I2C Request Callback");
        };

        let _subscription = i2c_driver.subscribe(&mut callback);

        let mut buf: [u8; 32] = [0; 32];

        println!("Creating the request");
        let len = self.ctx
            .get_request()
            .vendor_defined(0xB, &VENDOR_IDS[0], &[0x00, header.command as u8, 0x00], &mut buf);

            println!("buf: {:#x?}", buf);

        println!("Creating write buffer");
        let mut master_write_buffer = I2cBuffer::default();
        // Skip the first byte, as that is the destination address
        for (i, d) in buf[1..].iter().enumerate() {
            master_write_buffer[i] = *d;
        }
        let _dest_buffer = i2c_driver.init_buffer(&mut master_write_buffer);
        println!("  done");

        let _ = i2c_driver.write(DEST_ID as usize, len.unwrap() - 1);

        unsafe { syscalls::raw::yieldk() };
    }

    /// Gets the most recent response recieved until `request()` is called
    /// again.
    pub fn response(&mut self, buf: &mut [u8]) -> Option<(Header)> {
        println!("ManticoreHost: response");

        let i2c_driver;
        match self.i2c.init_driver() {
            Err(_) => {
                panic!("I2c init error");
            }
            Ok(driver) => {
                i2c_driver = driver;
            }
        }

        println!("Setting callback");
        let mut callback = |_, _| {
            println!("I2C Response Callback");
        };

        let _subscription = i2c_driver.subscribe(&mut callback);

        println!("Creating read buffer");
        let mut master_write_buffer = I2cBuffer::default();
        let dest_buffer;
        match i2c_driver.init_buffer(&mut master_write_buffer) {
            Err(_) => {
                panic!("I2c buffer init error");
            }
            Ok(buffer) => {
                dest_buffer = buffer;
            }
        }
        println!("  done");

        // Read 4 bytes for the SMBus header
        let _ = i2c_driver.read(DEST_ID as usize, 4);

        unsafe { syscalls::raw::yieldk() };

        println!("Finished first read");

        // Copy into a temp buffer
        let mut temp_buffer = [0; libtock::hmac::DEST_BUFFER_SIZE];
        dest_buffer.read_bytes(&mut temp_buffer[1..4]);

        for d in temp_buffer[0..4].iter() {
            println!("{:#x}", *d);
        }


        // Determine the full length
        let bytes = self.ctx.get_length(&temp_buffer);

        println!("Length: {:?}", bytes);

        let bytes = bytes.unwrap();

        // Read the full packet. The slave will re-send the data, so do
        // a full read
        let _ = i2c_driver.read(DEST_ID as usize, bytes);

        unsafe { syscalls::raw::yieldk() };

        println!("Finished second read");

        // Copy in the full packet, with space for the destination address
        dest_buffer.read_bytes(&mut temp_buffer[1..bytes]);

        println!("  Copied data");

        // Set the destination address as this isn't filled in the buffer from
        // the kernel
        temp_buffer[0] = MY_ID << 1;

        // Print the buffer
        for d in temp_buffer[0..bytes].iter() {
            println!("{:#x}", *d);
        }

        // Decode the response
        let (_msg_type, ret) = self.ctx.decode_packet(&temp_buffer[0..bytes]).unwrap();

        println!("ret: {:?}", ret);

        if ret[0] == 0x14 && ret[1] == 0x14 && ret[2] == 0 {
            let header = Header {
                // command: ret[3].into(),
                command: CommandType::FirmwareVersion,
                is_request: false,
            };
            for (i, d) in ret[4..].iter().enumerate() {
                buf[i] = *d;
            }

            return Some(header);
        }

        None
    }
}

impl<'a> HostPort for ManticoreHost<'a> {
    fn receive(&mut self) -> Result<&mut dyn HostRequest, net::Error> {
        println!("ManticoreHost: receive");

        let i2c_driver;
        match self.i2c.init_driver() {
            Err(_) => {
                panic!("Hmac init error");
            }
            Ok(driver) => {
                i2c_driver = driver;
            }
        }

        // let mut slave_read_buffer = I2cSlaveReadBuffer::default();
        // let slave_read_buffer_ret = hmac_driver.init_key_buffer(&mut slave_read_buffer);
        // if slave_read_buffer_ret.is_err() {
        // panic!("I2C Slave read buffer init error");
        // }

        unimplemented!()
    }
}

impl<'a> HostRequest for ManticoreHost<'a> {
    fn header(&self) -> Result<Header, net::Error> {
        println!("ManticoreHost: header");
        unimplemented!()
    }

    fn payload(&mut self) -> Result<&mut dyn Read, net::Error> {
        println!("ManticoreHost: payload");
        unimplemented!()
    }

    fn reply(&mut self, header: Header) -> Result<&mut dyn HostResponse, net::Error> {
        println!("ManticoreHost: reply");
        unimplemented!()
    }
}

impl<'a> HostResponse for ManticoreHost<'a> {
    fn sink(&mut self) -> Result<&mut dyn Write, net::Error> {
        println!("ManticoreHost: sink");
        unimplemented!()
    }

    fn finish(&mut self) -> Result<(), net::Error> {
        println!("ManticoreHost: finish");
        unimplemented!()
    }
}

#[alloc_error_handler]
unsafe fn alloc_error_handler(_: Layout) -> ! {
    println!("alloc_error_handler called");
    loop {
        syscalls::raw::yieldk();
    }
}

#[libtock::main]
async fn main() -> TockResult<()> {
    let mut drivers = libtock::retrieve_drivers()?;
    drivers.console.create_console();

    println!("Starting Manticore example");

    const NETWORKING: capabilities::Networking = capabilities::Networking {
        max_message_size: 1024,
        max_packet_size: 256,
        mode: capabilities::RotMode::Platform,
        roles: capabilities::BusRole::HOST,
    };

    const TIMEOUTS: capabilities::Timeouts = capabilities::Timeouts {
        regular: Duration::from_millis(30),
        crypto: Duration::from_millis(200),
    };

    const DEVICE_ID: device_id::DeviceIdentifier = device_id::DeviceIdentifier {
        vendor_id: 1,
        device_id: 2,
        subsys_vendor_id: 3,
        subsys_id: 4,
    };

    let identity = Identity::new(b"test version", b"random bits");
    let reset = Reset::new(0, Duration::from_millis(1));
    let rsa = Builder::new();

    let mut server = pa_rot::PaRot::new(pa_rot::Options {
        identity: &identity,
        reset: &reset,
        rsa: &rsa,
        device_id: DEVICE_ID,
        networking: NETWORKING,
        timeouts: TIMEOUTS,
    });

    // Create the Manticore host
    let mut port = ManticoreHost::new(
        MCTPSMBusContext::new(MY_ID, &MSG_TYPES, &VENDOR_IDS),
        drivers.i2c,
    );

    let mut arena = [0; 64];
    let mut arena = BumpArena::new(&mut arena);

    println!("Prepare to query firmware");
    // Prepare a request to push into the host.
    let header = Header {
        command: CommandType::FirmwareVersion,
        is_request: true,
    };

    port.request(header, &[0]);

    // Cause a delay before the I2C read
    for _i in 0..10000 {
        print!(".");
    }

    println!("Starting to process requests");
    let mut resp = [0; 64];

    let _header = port.response(&mut resp).unwrap();

    // server.process_request(&mut port, &mut arena).unwrap();

    // let i2c_driver = drivers.i2c.init_driver()?;

    // println!("Loading in request");
    // let mut request = I2cMasterWriteBuffer::default();
    // for (i, d) in header.as_bytes().iter().enumerate() {
    //     request[i] = *d;
    // }
    // let request = i2c_driver.init_key_buffer(&mut request)?;

    // host.request(&request);

    // // Prepare to receive a message.
    // let mut host_req = host.receive()?;
    // let header = host_req.header()?;
    // assert_eq!(header.command, CommandType::FirmwareVersion);
    // assert!(header.is_request);

    loop {
        unsafe { syscalls::raw::yieldk() };
    }
}
