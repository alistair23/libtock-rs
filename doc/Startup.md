Startup
=======

This document describes the `libtock_runtime` startup process, up until the
process binary's `main` starts executing.

## Step 1: `start` assembly

The first code to start executing is in a symbol called `start`, which is
written in handwritten assembly. These implementations are specific to each
architecture, and live in `runtime/asm`. This assembly does the following:

1. Checks the initial program counter value against the correct `start` address.
   This verifies the process was deployed at the direct address in non-volatile
   storage. This is necessary because `libtock-rs` apps are statically-linked,
   and an incorrect location would cause undefined behavior. If this check
   fails, an error may be reported (if the `low_level_debug` capsule is present)
   and the process terminates.
1. Moves the process break to make room for the stack, `.data`, and `.bss`. The
   process break is the top of the process-accessible RAM. The process break is
   initially moved to be shortly after the end of the `.bss` section (depending
   on alignment constraints).
1. Initialize the stack. The initial stack pointer value is provided by the
   linker script, which calculates it using a symbol called `STACK_MEMORY` in
   the `.stack_buffer` section.
1. Copies `.data` from non-volatile storage into RAM. The .data section contains
   read-write global variables (e.g. `static mut` values) that have nonzero
   initial values.
1. Zeroes out `.bss`. `.bss` contains read-write global variables that have zero
   initial values.
1. Calls `rust_start`.

## Step 2: `rust_start`

`rust_start` is the first Rust code to execute in a process. It is defined in
the `libtock_runtime::startup` module. It runs some higher-level initialization,
such as giving debug information (stack and heap addresses) to the kernel.
`rust_start` then calls `libtock_unsafe_main`.

## Step 3: `libtock_unsafe_main`

`libtock_unsafe_main` is a shim used to direct execution from `libtock_runtime`
to the process binary's `main` function. It is generated by the
`libtock_runtime::set_main!` macro. `libtock_unsafe_main` just calls the
user-provided `main` function.

## Step 4: `main`

At this point, the user's `main` starts executing, and `libtock_runtime` is no
longer in control. Note that unlike most Rust programs, `main` is expected to
*not* return. Process binaries can loop forever or use the `exit` system call to
terminate when they are done executing (which may be done as the last statement
of `main`).

## Appendix: Why `#![no_main]`?

Writing a `#![no_std]` `bin` crate currently requires using either `#![no_main]`
or the `start` unstable feature. Because we want to move `libtock-rs` to stable
Rust eventually (hopefully soon after `asm` is stabilized), `libtock-rs` expects
process binaries to be `#![no_main]`.