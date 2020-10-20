//! This is a featured CTAP example
//! WARNING! This currently uses unsound crypto operations
//! This is only a demo and should not be used in real enviroments
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::vec::Vec;
use core::alloc::Layout;
use core::time::Duration;
use libtock::i2c::I2cMasterWriteBuffer;
use libtock::println;
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

pub struct ManticoreHost {
    i2c: I2cDriverFactory,
}

impl ManticoreHost {
    pub fn new(i2c: I2cDriverFactory) -> Self {
        Self { i2c }
    }

    /// Schedules a new request to be received, with the given request parts.
    ///
    /// Calling this function will make `receive()` start working; otherwise,
    /// it will assert that the port is disconnected.
    pub fn request(&mut self, header: Header, message: &[u8]) {
        println!("ManticoreHost: request");
        // header.write_bytes();
    }

    /// Gets the most recent response recieved until `request()` is called
    /// again.
    pub fn response(&self) -> Option<(Header, &[u8])> {
        println!("ManticoreHost: response");
        unimplemented!()
    }
}

impl HostPort for ManticoreHost {
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

        let mut slave_read_buffer = I2cSlaveReadBuffer::default();
        let slave_read_buffer_ret = hmac_driver.init_key_buffer(&mut slave_read_buffer);
        if slave_read_buffer_ret.is_err() {
            panic!("I2C Slave read buffer init error");
        }

        unimplemented!()
    }
}

impl HostRequest for ManticoreHost {
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

impl HostResponse for ManticoreHost {
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

    let mut port = ManticoreHost::new(drivers.i2c);

    let mut arena = [0; 64];
    let mut arena = BumpArena::new(&mut arena);

    println!("Starting to process requests");

    server.process_request(&mut port, &mut arena).unwrap();

    let (header, mut resp) = port.response().unwrap();

    // let options = pa_rot::Options

    // let mut pa_rot = pa_rot::PaRot::new(Non)

    // let i2c_driver = drivers.i2c.init_driver()?;

    // // Prepare a request to push into the host.
    // let header = Header {
    //     command: CommandType::FirmwareVersion,
    //     is_request: true,
    // };

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
