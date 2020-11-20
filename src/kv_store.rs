use core::ops::{Deref, DerefMut};
use crate::callback::CallbackSubscription;
use crate::callback::Consumer;
use crate::result::TockResult;
use crate::syscalls;
use core::marker::PhantomData;
use libtock_core::shared_memory::SharedMemory;

const DRIVER_NUMBER: usize = 0x50003;

pub const KEY_BUFFER_SIZE: usize = 32;
pub const VALUE_BUFFER_SIZE: usize = 64;

mod command_nr {
    pub const APPEND_KEY: usize = 0;
    pub const GET_KEY: usize = 1;
    pub const INVALIDATE_KEY: usize = 2;
    pub const GARBAGE_COLLECT: usize = 3;
}

mod subscribe_nr {
    pub const SUBSCRIBE_CALLBACK: usize = 0;
}

mod allow_nr {
    pub const KEY: usize = 0;
    pub const VALUE: usize = 1;
}

#[non_exhaustive]
pub struct KVStoreDriverFactory;

impl KVStoreDriverFactory {
    pub fn init_driver(&mut self) -> TockResult<KVStoreDriver> {
        let kvstore = KVStoreDriver {
            lifetime: PhantomData,
        };
        Ok(kvstore)
    }
}

struct KVStoreEventConsumer;

impl<CB: FnMut(usize, usize)> Consumer<CB> for KVStoreEventConsumer {
    fn consume(callback: &mut CB, _: usize, _: usize, _: usize) {
        callback(0, 0);
    }
}

#[derive(Debug)]
pub struct KVStoreKeyBuffer {
    buffer: [u8; KEY_BUFFER_SIZE],
}

impl KVStoreKeyBuffer {
    pub fn new(buf: [u8; KEY_BUFFER_SIZE]) -> Self {
        KVStoreKeyBuffer { buffer: buf }
    }
}

impl Default for KVStoreKeyBuffer {
    fn default() -> Self {
        KVStoreKeyBuffer {
            buffer: [0; KEY_BUFFER_SIZE],
        }
    }
}

impl Deref for KVStoreKeyBuffer {
    type Target = [u8; KEY_BUFFER_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for KVStoreKeyBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

pub struct KVStoreValueBuffer {
    pub buffer: [u8; VALUE_BUFFER_SIZE],
}

impl KVStoreValueBuffer {
    pub fn new(buf: [u8; VALUE_BUFFER_SIZE]) -> Self {
        KVStoreValueBuffer { buffer: buf }
    }
}

impl Default for KVStoreValueBuffer {
    fn default() -> Self {
        KVStoreValueBuffer {
            buffer: [0; VALUE_BUFFER_SIZE],
        }
    }
}

pub struct KVStoreDriver<'a> {
    lifetime: PhantomData<&'a ()>,
}

impl<'a> KVStoreDriver<'a> {
    pub fn init_key_buffer(&self, buffer: &'a mut KVStoreKeyBuffer) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::KEY, &mut buffer.buffer).map_err(Into::into)
    }

    pub fn init_value_buffer(
        &self,
        buffer: &'a mut KVStoreValueBuffer,
    ) -> TockResult<SharedMemory> {
        syscalls::allow(DRIVER_NUMBER, allow_nr::VALUE, &mut buffer.buffer).map_err(Into::into)
    }

    pub fn subscribe<CB: FnMut(usize, usize)>(
        &self,
        callback: &'a mut CB,
    ) -> TockResult<CallbackSubscription> {
        syscalls::subscribe::<KVStoreEventConsumer, _>(
            DRIVER_NUMBER,
            subscribe_nr::SUBSCRIBE_CALLBACK,
            callback,
        )
        .map_err(Into::into)
    }
    pub fn append_key(&self, key_len: usize) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::APPEND_KEY, key_len, 0)?;
        Ok(())
    }

    pub fn get_key(&self, key_len: usize) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::GET_KEY, key_len, 0)?;
        Ok(())
    }

    pub fn invalidate_key(&self, key_len: usize) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::INVALIDATE_KEY, key_len, 0)?;
        Ok(())
    }

    pub fn garbage_collect(&self) -> TockResult<()> {
        syscalls::command(DRIVER_NUMBER, command_nr::GARBAGE_COLLECT, 0, 0)?;
        Ok(())
    }
}
