#![no_std]
use libtock::memop::*;
use core::fmt::Write;
use libtock::result::TockResult;
use libtock::syscalls::raw::*;

#[libtock::main]
async fn main() -> TockResult<()> {
    let drivers = libtock::retrieve_drivers()?;

    let mut console = drivers.console.create_console();

    writeln!(console, "Starting PMP test")?;

    let address = get_mem_start() as usize;
    let address_end = get_mem_end() as usize;

    writeln!(console, "  app_start: 0x20430020")?;
    writeln!(console, "  mem_start/mem_len: {:x}/{:x}", address, address_end)?;

/************* READ TEST MEMORY - PASSING ****************/
    writeln!(console, "")?;
    writeln!(console, "Starting memory read test: Reading from {:x} to {:x}", address, address_end)?;

    for addr in address..address_end {
        let ptr = addr as *mut u32;

        if (addr % 0x200) == 0 {
            writeln!(console, "    reading from: 0x{:x}", addr)?;
        }

        unsafe {
            core::ptr::read_volatile(ptr);
        }
    }

    writeln!(console, "  Finished memory read")?;

/************* READ TEST FLASH - PASSING ****************/
    let flash = get_flash_start() as usize;
    let flash_end = get_flash_end() as usize;

    writeln!(console, "Starting flash read test: Reading from {:x} to {:x}", flash, flash_end)?;

    for addr in flash..flash_end {
        let ptr = addr as *mut u32;

        if (addr % 0x1000) == 0 {
            writeln!(console, "    reading from: 0x{:x}", addr)?;
        }

        unsafe {
            core::ptr::read_volatile(ptr);
        }
    }

    writeln!(console, "  Finished flash read")?;

/************* WRITE TEST MEMORY WITH INC - PASSING ****************/
    let brk_og = get_brk() as usize;
    increment_brk(0x400);
    let brk = get_brk() as usize;

    writeln!(console, "Incremented BRK from: 0x{:x} to 0x{:x}", brk_og, brk)?;

    writeln!(console, "Increment BRK to 0x{:x}", brk)?;

    writeln!(console, "Starting memory inc write test: Writing to 0x{:x} to 0x{:x}", brk_og, brk)?;

    for addr in brk_og..brk {
        let ptr = addr as *mut u32;

        if (addr % 0x100) == 0 {
            writeln!(console, "    writing to: 0x{:x}", addr)?;
        }

        unsafe {
            core::ptr::write_volatile(ptr, 0xDEADBEEF);
        }
    }

    writeln!(console, "  Finished brk inc write")?;

/************* READ TESTS - FAILING ****************/
    // writeln!(console, "")?;
    // writeln!(console, "Starting memory read test: Reading from invalid address {:x} to {:x}", address_end, address_end + 0x100)?;

    // for addr in address_end..(address_end + 0x100) {
    //     let ptr = addr as *mut u32;

    //     writeln!(console, "    reading from: 0x{:x}", addr)?;

    //     unsafe {
    //         core::ptr::read_volatile(ptr);
    //     }
    // }

    // writeln!(console, "  Finished memory read")?;

/************* READ TEST FLASH - FAILING ****************/
    // let flash_end = get_flash_end() as usize;

    // writeln!(console, "Starting flash read test: Reading from invalid address {:x} to {:x}", flash_end, flash_end + 0x100)?;

    // for addr in flash_end..flash_end + 0x100 {
    //     let ptr = addr as *mut u32;

    //     writeln!(console, "    reading from: 0x{:x}", addr)?;

    //     unsafe {
    //         core::ptr::read_volatile(ptr);
    //     }
    // }

    // writeln!(console, "  Finished flash read")?;


    writeln!(console, "Done!")?;

    loop {
        unsafe{ yieldk(); }
    }
}


/******** This diff was applied to Tock for testing ********/
/*
diff --git a/boards/hifive1/layout.ld b/boards/hifive1/layout.ld
index 207ff939..1767b84c 100644
--- a/boards/hifive1/layout.ld
+++ b/boards/hifive1/layout.ld
@@ -8,7 +8,7 @@ MEMORY
 {
   rom (rx)  : ORIGIN = 0x20400000, LENGTH = 0x30000
   prog (rx) : ORIGIN = 0x20430000, LENGTH = 512M-0x430000
-  ram (rwx) : ORIGIN = 0x80000000, LENGTH = 16K
+  ram (rwx) : ORIGIN = 0x80000000, LENGTH = 2M
 }
 
 MPU_MIN_ALIGN = 1K;
diff --git a/boards/hifive1/src/main.rs b/boards/hifive1/src/main.rs
index ed6278e4..ee86d7f8 100644
--- a/boards/hifive1/src/main.rs
+++ b/boards/hifive1/src/main.rs
@@ -33,12 +33,12 @@ const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultRespons
 
 // RAM to be shared by all application processes.
 #[link_section = ".app_memory"]
-static mut APP_MEMORY: [u8; 5 * 1024] = [0; 5 * 1024];
+static mut APP_MEMORY: [u8; 16 * 1024] = [0; 16 * 1024];
 
 /// Dummy buffer that causes the linker to reserve enough space for the stack.
 #[no_mangle]
 #[link_section = ".stack_buffer"]
-pub static mut STACK_MEMORY: [u8; 0x800] = [0; 0x800];
+pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];
 
 /// A structure representing this platform that holds references to all
 /// capsules for this platform. We've included an alarm and console.
diff --git a/chips/e310x/src/chip.rs b/chips/e310x/src/chip.rs
index 9fe4fd96..6198b623 100644
--- a/chips/e310x/src/chip.rs
+++ b/chips/e310x/src/chip.rs
@@ -13,12 +13,14 @@ use crate::uart;
 
 pub struct E310x {
     userspace_kernel_boundary: rv32i::syscall::SysCall,
+    pmp: rv32i::pmp::PMPConfig
 }
 
 impl E310x {
     pub unsafe fn new() -> E310x {
         E310x {
             userspace_kernel_boundary: rv32i::syscall::SysCall::new(),
+            pmp: rv32i::pmp::PMPConfig::new(4),
         }
     }
 
@@ -30,12 +32,12 @@ impl E310x {
 }
 
 impl kernel::Chip for E310x {
-    type MPU = ();
+    type MPU = rv32i::pmp::PMPConfig;
     type UserspaceKernelBoundary = rv32i::syscall::SysCall;
     type SysTick = ();
 
     fn mpu(&self) -> &Self::MPU {
-        &()
+        &self.pmp
     }
 
     fn systick(&self) -> &Self::SysTick {
*/

/******** With this diff to QEMU ********/
/*
diff --git a/hw/riscv/sifive_e.c b/hw/riscv/sifive_e.c
index 8a6b0348df..672c659948 100644
--- a/hw/riscv/sifive_e.c
+++ b/hw/riscv/sifive_e.c
@@ -72,7 +72,7 @@ static const struct MemmapEntry {
     [SIFIVE_E_QSPI2] =    { 0x10034000,     0x1000 },
     [SIFIVE_E_PWM2] =     { 0x10035000,     0x1000 },
     [SIFIVE_E_XIP] =      { 0x20000000, 0x20000000 },
-    [SIFIVE_E_DTIM] =     { 0x80000000,     0x4000 }
+    [SIFIVE_E_DTIM] =     { 0x80000000,   0x800000 }
 };
 
 static void riscv_sifive_e_init(MachineState *machine)
*/
