// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>

//! GPIO Driver.

use tock_registers::{
    interfaces::{ReadWriteable, Writeable},
    register_bitfields, register_structs,
    registers::ReadWrite,
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// GPIO registers.
//
// Descriptions taken from
// - https://datasheets.raspberrypi.org/bcm2711/bcm2711-peripherals.pdf
register_bitfields! {
    u32,

    /// GPIO Function Select 1
    GPFSEL1 [

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100  // PL011 UART TX
        ],

        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100  // PL011 UART RX

        ]
    ],

    /// GPIO Function Select 4
    GPFSEL4 [
        /// Pin 42
        FSEL42 OFFSET(6) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
        ],
    ],

    GPSET1 [
        /// Pin 42
        SET14 OFFSET(10) NUMBITS(1) [
            Keep = 0b0,
            Set = 0b1,
        ],
    ],

    GPCLR1 [
        /// Pin 42
        CLR14 OFFSET(10) NUMBITS(1) [
            Keep = 0b0,
            Clear = 0b1,
        ],
    ],


    /// GPIO Pull-up / Pull-down Register 0
    ///
    /// BCM2711 only.
    GPIO_PUP_PDN_CNTRL_REG0 [
        /// Pin 15
        GPIO_PUP_PDN_CNTRL15 OFFSET(30) NUMBITS(2) [
            NoResistor = 0b00,
            PullUp = 0b01
        ],

        /// Pin 14
        GPIO_PUP_PDN_CNTRL14 OFFSET(28) NUMBITS(2) [
            NoResistor = 0b00,
            PullUp = 0b01
        ]
    ]
}

register_structs! {
    GpioRegisterBlock {
        (0x00 => _reserved1),
        (0x04 => gpfsel1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => gpfsel2: ReadWrite<u32>),
        (0x0C => gpfsel3: ReadWrite<u32>),
        (0x10 => gpfsel4: ReadWrite<u32, GPFSEL4::Register>),
        (0x14 => gpfsel5: ReadWrite<u32>),
        (0x18 => _reserved2),
        (0x1C => gpset0: ReadWrite<u32>),
        (0x20 => gpset1: ReadWrite<u32, GPSET1::Register>),
        (0x24 => _reserved3),
        (0x28 => gpclr0: ReadWrite<u32>),
        (0x2C => gpclr1: ReadWrite<u32, GPCLR1::Register>),
        (0x30 => _reserved4),
        (0x34 => gplev0: ReadWrite<u32>),
        (0x38 => gplev1: ReadWrite<u32>),
        (0x3C => _reserved5),
        (0x40 => gpeds0: ReadWrite<u32>),
        (0x44 => gpeds1: ReadWrite<u32>),
        (0x48 => _reserved6),
        (0x4C => gpren0: ReadWrite<u32>),
        (0x50 => gpren1: ReadWrite<u32>),
        (0x54 => _reserved7),
        (0x58 => gpfen0: ReadWrite<u32>),
        (0x5C => gpfen1: ReadWrite<u32>),
        (0x60 => _reserved8),
        (0x64 => gphen0: ReadWrite<u32>),
        (0x68 => gphen1: ReadWrite<u32>),
        (0x6C => _reserved9),
        (0x70 => gplen0: ReadWrite<u32>),
        (0x74 => gplen1: ReadWrite<u32>),
        (0x78 => _reserved10),
        (0x7C => gparen0: ReadWrite<u32>),
        (0x80 => gparen1: ReadWrite<u32>),
        (0x84 => _reserved11),
        (0x88 => gpafen0: ReadWrite<u32>),
        (0x8C => gpafen1: ReadWrite<u32>),
        (0x90 => _reserved12),

        (0xE4 => gpio_pup_pdn_cntrl_reg0: ReadWrite<u32, GPIO_PUP_PDN_CNTRL_REG0::Register>),
        (0xE8 => gpio_pup_pdn_cntrl_reg1: ReadWrite<u32>),
        (0xEC => gpio_pup_pdn_cntrl_reg2: ReadWrite<u32>),
        (0xF0 => gpio_pup_pdn_cntrl_reg3: ReadWrite<u32>),
        (0xF4 => @END),
    }
}

pub struct Gpio {
    r: &'static mut GpioRegisterBlock,
}

const GPIO_START: usize = 0xFE20_0000; // Based on section 1.2 of manual
                                       //
impl Gpio {
    pub fn new() -> Self {
        Gpio {
            r: unsafe { &mut *(GPIO_START as *mut GpioRegisterBlock) }
        }
    }
    pub fn pin_42_config_output(&mut self) {
        self.r.gpfsel4.modify(GPFSEL4::FSEL42::Output);
    }

    pub fn pin_42_set(&mut self) {
        self.r.gpset1.write(GPSET1::SET14::Set);
    }

    pub fn pin_42_clr(&mut self) {
        self.r.gpclr1.write(GPCLR1::CLR14::Clear);
    }

    /// Disable pull-up/down on pins 14 and 15.
    fn disable_pud_14_15_bcm2711(&mut self) {
        self.r.gpio_pup_pdn_cntrl_reg0.modify(
            GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::PullUp
                + GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::PullUp,
        );
    }

    /// Map PL011 UART as standard output.
    ///
    /// TX to pin 14
    /// RX to pin 15
    pub fn map_pl011_uart(&mut self) {
        // Select the UART on pins 14 and 15.
        self.r.gpfsel1
            .modify(GPFSEL1::FSEL15::AltFunc0 + GPFSEL1::FSEL14::AltFunc0);

        self.disable_pud_14_15_bcm2711();
    }
}
