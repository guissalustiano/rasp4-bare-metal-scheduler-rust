#![no_main]
#![no_std]

#![feature(const_mut_refs)]

pub mod timer;
pub mod gpio;
pub mod pl011_uart;


use aarch64_cpu::asm::nop;
use gpio::Gpio;
use timer::spin_for;

use core::{panic::PanicInfo, time::Duration};
use tock_registers::interfaces::Readable;

// https://datasheets.raspberrypi.com/bcm2711/bcm2711-peripherals.pdf
const START:            usize = 0xFE00_0000; // Based on section 1.2 of manual
const GPIO_OFFSET:      usize = 0x0020_0000; // Based on section 5.2 of manual, also check that
                                             // 0x7enn_nnnn is mapped to 0xFEnn_nnnn
const GPIO_START:       usize = START + GPIO_OFFSET;

#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    if get_cpu_id() != 0 {
        aarch64_cpu::asm::wfe();
    }

    let mut gpio = Gpio::new();

    // Set GPIO 42 with GPFSEL4[8:6] = 0b001
    gpio.pin_42_config_output();
    loop {
        // Set GPIO 42 to HIGH with GPSET1
        gpio.pin_42_set();

        // Wait
        spin_for(Duration::from_millis(500));

        // Set GPIO 42 to LOW with GPCLR[42-32]
        gpio.pin_42_clr();

        // Wait
        spin_for(Duration::from_millis(500));
    }
}

fn get_cpu_id() -> u64 {
    aarch64_cpu::registers::MPIDR_EL1.get() & 0b11
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    unsafe {
        // Set GPIO 42 with GPFSEL4[8:6] = 0b001
        core::ptr::write_volatile((GPIO_START + 0x10) as *mut u32, 0b001 << 6);
        loop {
            // Set GPIO 42 to HIGH with GPSET1
            core::ptr::write_volatile((GPIO_START + 0x20) as *mut u32, 1 << (42-32));

            // Wait
            for _ in 0..5000000 {
                nop();
            }

            // Set GPIO 42 to LOW with GPCLR[42-32]
            core::ptr::write_volatile((GPIO_START + 0x02c) as *mut u32, 1 << (42-32));

            // Wait
            for _ in 0..5000000 {
                nop();
            }
        }
    }
}
