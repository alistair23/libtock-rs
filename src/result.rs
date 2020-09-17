use core::fmt;

pub use libtock_core::result::*;

pub type TockResult<T> = Result<T, TockError>;

#[derive(Copy, Clone, Debug)]
pub enum TockError {
    Subscribe(SubscribeError),
    Command(CommandError),
    Allow(AllowError),
    Format,
    Other(OtherError),
}

impl From<SubscribeError> for TockError {
    fn from(subscribe_error: SubscribeError) -> Self {
        TockError::Subscribe(subscribe_error)
    }
}

impl From<CommandError> for TockError {
    fn from(command_error: CommandError) -> Self {
        TockError::Command(command_error)
    }
}

impl From<AllowError> for TockError {
    fn from(allow_error: AllowError) -> Self {
        TockError::Allow(allow_error)
    }
}

impl From<fmt::Error> for TockError {
    fn from(fmt::Error: fmt::Error) -> Self {
        TockError::Format
    }
}

#[derive(Copy, Clone, Debug)]
pub enum OtherError {
    ButtonsDriverInvalidState,
    GpioDriverInvalidState,
    TimerDriverDurationOutOfRange,
    TimerDriverErroneousClockFrequency,
    DriversAlreadyTaken,
    OutOfRange,
}

impl From<OtherError> for TockError {
    fn from(other: OtherError) -> Self {
        TockError::Other(other)
    }
}

pub struct OutOfRangeError;

impl From<OutOfRangeError> for TockError {
    fn from(_: OutOfRangeError) -> Self {
        TockError::Other(OtherError::OutOfRange)
    }
}
