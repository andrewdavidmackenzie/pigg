use crate::hw::{InputPull, PinDescription, PinFunction};

/// This module codifies the descriptions if the Raspberry Pi GPIO hardware
/// exposed pins, including multiple options (functions) available for some pins
/// via software configuration.
///
/// In general, it has been harvested from the
/// [official Raspberry Pi docs](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#gpio-and-the-40-pin-header)
/// , although sometimes augmented with other sources.
///
/// The default Pullup/Pulldown settings are taking from the "BCM2711 ARM Peripherals" document
///
/// These pin descriptions are valid for Raspberry Pi Models B+, 2B, Zero, 3B, 3B+,
/// 4B, Zero W, Zero2 W, 5
///
/// For SPI interface description, see [here](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
/// "Raspberry Pi Zero, 1, 2 and 3 have three SPI controllers:"
///
/// "Raspberry Pi 4, 400 and Compute Module 4 there are four additional SPI buses: SPI3 to SPI6,
/// each with two hardware chip selects. These extra SPI buses are available via alternate function
/// assignments on certain GPIO pins. For more information, see the BCM2711 Arm peripherals
/// datasheet."

pub const PIN_1: PinDescription = PinDescription {
    board_pin_number: 1,
    bcm_pin_number: None,
    name: "3V3",
    options: &[PinFunction::Power3V3],
};

pub const PIN_2: PinDescription = PinDescription {
    board_pin_number: 2,
    bcm_pin_number: None,
    name: "5V",
    options: &[PinFunction::Power5V],
};

/// "Pins GPIO2 and GPIO3 have fixed pull-up resistors"
pub const PIN_3: PinDescription = PinDescription {
    board_pin_number: 3,
    bcm_pin_number: Some(2),
    name: "GPIO2",
    options: &[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO PinFunction::I2C1_SDA,
        // TODO ALT1: SMI SA3
        // TODO ALT2: DPI VSYNC
        // TODO ALT3: AVEOUT_VSYNC
        // TODO ALT4: AVEIN_VSYNC
    ],
};

pub const PIN_4: PinDescription = PinDescription {
    board_pin_number: 4,
    bcm_pin_number: None,
    name: "5V",
    options: &[PinFunction::Power5V],
};

/// "Pins GPIO2 and GPIO3 have fixed pull-up resistors"
pub const PIN_5: PinDescription = PinDescription {
    board_pin_number: 5,
    bcm_pin_number: Some(3),
    name: "GPIO3",
    options: &[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO PinFunction::I2C1_SCL,
        // TODO ALT1: SMI SA2
        // TODO ALT2: DPI_HSYNC
        // TODO ALT3: AVEOUT_HSYNC
        // TODO ALT4: AVEIN_HSYNC
    ],
};

pub const PIN_6: PinDescription = PinDescription {
    board_pin_number: 6,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
};

pub const PIN_7: PinDescription = PinDescription {
    board_pin_number: 7,
    bcm_pin_number: Some(4),
    name: "GPIO4",
    options: &[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO PinFunction::I2C3_SDA, // TODO is this correct?
        // TODO PinFunction::GPCLK0,
        // TODO ALT1: SMI SA1
        // TODO ALT2: DPI D0
        // TODO ALT3: AVEOUT_VID0
        // TODO ALT4: AVEIN_VID0
        // TODO ALT5: JTAG_TDI
    ],
};

pub const PIN_8: PinDescription = PinDescription {
    board_pin_number: 8,
    bcm_pin_number: Some(14),
    name: "GPIO14",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::UART0_TXD,
        // TODO ALT1: SMI SD6
        // TODO ALT2: DSI D10
        // TODO ALT3: AVEOUT_VID10
        // TODO ALT4: AVEIN_VID10
        // TODO ALT5: UART1_TXD
    ],
};

pub const PIN_9: PinDescription = PinDescription {
    board_pin_number: 9,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
};

pub const PIN_10: PinDescription = PinDescription {
    board_pin_number: 10,
    bcm_pin_number: Some(15),
    name: "GPIO15",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::UART0_RXD,
        // TODO ALT1: SMI SD7
        // TODO ALT2: DPI D11
        // TODO ALT3: AVEOUT VID11
        // TODO ALT4: AVEIN VID11
        // TODO ALT5: UART1_RXD
    ],
};

pub const PIN_11: PinDescription = PinDescription {
    board_pin_number: 11,
    bcm_pin_number: Some(17),
    name: "GPIO17",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::SPI1_CE1_N,
        // TODO ALT0: FL1
        // TODO ALT1: SMI SD9
        // TODO ALT2: DPI D13
        // TODO ALT3: UART0 RTS
        // TODO ALT5: UART1 RTS
    ],
};

pub const PIN_12: PinDescription = PinDescription {
    board_pin_number: 12,
    bcm_pin_number: Some(18),
    name: "GPIO18",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::PCM_CLK,    // ALT0
        // TODO ALT1: SMI SD10
        // TODO ALT2: DPI D14
        // TODO ALT3: I2C SLA/MOSI ??
        // TODO PinFunction::SPI1_CE0_N, // ALT4
        // TODO PinFunction::PWM0,       // ALT5
    ],
};

pub const PIN_13: PinDescription = PinDescription {
    board_pin_number: 13,
    bcm_pin_number: Some(27),
    name: "GPIO27",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::PWM1, // TODO is this correct?
        // TODO ALT0: SD0 DAT3
        // TODO ALT1: TE1
        // TODO ALT2: DPI D23
        // TODO ALT3: SD1 DAT3
        // TODO ALT4: JTA TMS
    ],
};

pub const PIN_14: PinDescription = PinDescription {
    board_pin_number: 14,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
};

pub const PIN_15: PinDescription = PinDescription {
    board_pin_number: 15,
    bcm_pin_number: Some(22),
    name: "GPIO22",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::I2C6_SDA, // TODO is this correct
        // TODO ALT0: SD0 CLK
        // TODO ALT1: SMI SD14
        // TODO ALT2: DPI D18
        // TODO ALT3: SD1 CLK
        // TODO ALT4: JTA TRST
    ],
};

pub const PIN_16: PinDescription = PinDescription {
    board_pin_number: 16,
    bcm_pin_number: Some(23),
    name: "GPIO23",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::I2C6_SCL, // TODO is this correct?
        // TODO ALT0: SD0 CMD
        // TODO ALT1: SMI SD15
        // TODO ALT2: DPI D19
        // TODO ALT3: SD1 CMD
        // TODO ALT4: JTA RTCK
    ],
};

pub const PIN_17: PinDescription = PinDescription {
    board_pin_number: 17,
    bcm_pin_number: None,
    name: "3V3",
    options: &[PinFunction::Power3V3],
};

pub const PIN_18: PinDescription = PinDescription {
    board_pin_number: 18,
    bcm_pin_number: Some(24),
    name: "GPIO24",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::PWM0, // TODO is this correct?
        // TODO ALT0: SD0 DAT0
        // TODO ALT1: SMI SD16
        // TODO ALT2: DPI D20
        // TODO ALT3: SD1 DAT0
        // TODO ALT4: JTA TDO
    ],
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
pub const PIN_19: PinDescription = PinDescription {
    board_pin_number: 19,
    bcm_pin_number: Some(10),
    name: "GPIO10",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::SPI0_MOSI, // ALT0
        // TODO PinFunction::PWM1,      // TODO is this correct?
        // TODO ALT1: SMI SD2
        // TODO ALT2: DPI D6
        // TODO ALT3: AVEOUT VID6
        // TODO ALT4: AVEIN VID6
    ],
};

pub const PIN_20: PinDescription = PinDescription {
    board_pin_number: 20,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
pub const PIN_21: PinDescription = PinDescription {
    board_pin_number: 21,
    bcm_pin_number: Some(9),
    name: "GPIO9",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::I2C4_SCL, // TODO is this correct?
        // TODO PinFunction::SPI0_MISO, // ALT0
        // TODO ALT1: SMI SD1
        // TODO ALT2: DPI D5
        // TODO ALT3: AVEOUT VID5
        // TODO ALT4: AVEIN VID5
    ],
};

pub const PIN_22: PinDescription = PinDescription {
    board_pin_number: 22,
    bcm_pin_number: Some(25),
    name: "GPIO25",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: SD0 DAT1
        // TODO ALT1: SMI SD17
        // TODO ALT2: DPI D21
        // TODO ALT3: SD1 DAT1
        // TODO ALT4: JTAG TCK
    ],
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
pub const PIN_23: PinDescription = PinDescription {
    board_pin_number: 23,
    bcm_pin_number: Some(11),
    name: "GPIO11",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::SPI0_SCLK, // ALT0
        // TODO ALT1: SMI SD3
        // TODO ALT2: DPI D7
        // TODO ALT3: AVEOUT VID7
        // TODO ALT4: AVEIN VID7
    ],
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
pub const PIN_24: PinDescription = PinDescription {
    board_pin_number: 24,
    bcm_pin_number: Some(8),
    name: "GPIO8",
    options: &[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO PinFunction::SPI0_CE0_N, // ALT0
        // TODO PinFunction::I2C4_SDA,   // TODO is this correct?
        // TODO ALT1: SMI SD0
        // TODO ALT2: DPI D4
        // TODO ALT3: AVEOUT VID4
        // TODO ALT4: AVEIN VID4
    ],
};

pub const PIN_25: PinDescription = PinDescription {
    board_pin_number: 25,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
pub const PIN_26: PinDescription = PinDescription {
    board_pin_number: 26,
    bcm_pin_number: Some(7),
    name: "GPIO7",
    options: &[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO PinFunction::SPI0_CE1_N, // ALT0
        // TODO ALT1: SMI SWE_N / SRW_N
        // TODO ALT2: DPI D3
        // TODO ALT3: AVEOUT VID3
        // TODO ALT4: AVEIN VID3
    ],
};

pub const PIN_27: PinDescription = PinDescription {
    board_pin_number: 27,
    bcm_pin_number: None,
    name: "GPIO0", // EEPROM ID SCL
    options: &[
        // TODO PinFunction::I2C_EEPROM_ID_SD, // Is this ALT0 or the main function?
        // TODO ALT0: I2C0 SDA (I suspect is the main function for talking to EEPROM)
        // TODO ALT1: SMI SA5
        // TODO ALT2: DPI CLK
        // TODO ALT3: AVEOUT VCLK
        // TODO ALT4: AVEIN VCLK
    ],
};

pub const PIN_28: PinDescription = PinDescription {
    board_pin_number: 28,
    bcm_pin_number: None,
    name: "GPIO1", // EEPROM ID SCL
    options: &[
        // TODO PinFunction::I2C_EEPROM_ID_SC, // Is this ALT0 or the main function?
        // TODO ALT0: I2C0 SDL (I suspect is the main function for talking to EEPROM)
        // TODO ALT1: SMI SA4
        // TODO ALT2: DPI DEN
        // TODO ALT3: AVEOUT DSYNC
        // TODO ALT4: AVEIN DSYNC
    ],
};

pub const PIN_29: PinDescription = PinDescription {
    board_pin_number: 29,
    bcm_pin_number: Some(5),
    name: "GPIO5",
    options: &[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO PinFunction::I2C3_SCL, // TODO is this correct
        // TODO PinFunction::GPCLK1, // ALT0
        // TODO ALT1: SMI SA0
        // TODO ALT2: DPI D1
        // TODO ALT3: AVEOUT VID1
        // TODO ALT4: AVEIN VID1
        // TODO ALT5: JTA TDO
    ],
};

pub const PIN_30: PinDescription = PinDescription {
    board_pin_number: 30,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
};

pub const PIN_31: PinDescription = PinDescription {
    board_pin_number: 31,
    bcm_pin_number: Some(6),
    name: "GPIO6",
    options: &[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO PinFunction::GPCLK2, // ALT0
        // TODO ALT1: SMI SOE_N / SE
        // TODO ALT2: DPI D2
        // TODO ALT3: AVEOUT VID2
        // TODO ALT4: AVEIN VID2
        // TODO ALT5: JTA RTCK
    ],
};

pub const PIN_32: PinDescription = PinDescription {
    board_pin_number: 32,
    bcm_pin_number: Some(12),
    name: "GPIO12",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::I2C5_SDA, // TODO is this correct?
        // TODO PinFunction::PWM0,     // TODO should this be PWM0 according to pinout.xyz
        // TODO ALT1: SMI SD4
        // TODO ALT2: DPI D8
        // TODO ALT3: AVEOUT VID8
        // TODO ALT4: AVEIN VID8
        // TODO ALT5: JTA TMS
    ],
};

pub const PIN_33: PinDescription = PinDescription {
    board_pin_number: 33,
    bcm_pin_number: Some(13),
    name: "GPIO13",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::I2C5_SCL, // TODO is this correct
        // TODO PinFunction::PWM1,     // ALT0
        // TODO ALT1: SMI SD5
        // TODO ALT2: DPI D9
        // TODO ALT3: AVEOUT VID9
        // TODO ALT4: AVEIN VID9
        // TODO ALT5: JTA TCK
    ],
};

pub const PIN_34: PinDescription = PinDescription {
    board_pin_number: 34,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
};

pub const PIN_35: PinDescription = PinDescription {
    board_pin_number: 35,
    bcm_pin_number: Some(19),
    name: "GPIO19",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::PCM_FS,    // ALT0
        // TODO ALT1: SMI SD11
        // TODO ALT2: DPI D15
        // TODO ALT3: I2CSL SCL
        // TODO PinFunction::SPI1_MISO, // ALT4
        // TODO ALT5: PWM1
    ],
};

pub const PIN_36: PinDescription = PinDescription {
    board_pin_number: 36,
    bcm_pin_number: Some(16),
    name: "GPIO16",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: FL0
        // TODO ALT1: SMI SD8
        // TODO ALT2: DPI D12
        // TODO PinFunction::SPI1_CE2_N, // ALT4
        // TODO ALT3: UART0 CTS
        // TODO ALT5: UART1 CTS
    ],
};

pub const PIN_37: PinDescription = PinDescription {
    board_pin_number: 37,
    bcm_pin_number: Some(26),
    name: "GPIO26",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: SD0 DAT2
        // TODO ALT1: TE0
        // TODO ALT2: DPI D22
        // TODO ALT3: SD1 DAT2
        // TODO ALT4: JTA TDI
    ],
};

pub const PIN_38: PinDescription = PinDescription {
    board_pin_number: 38,
    bcm_pin_number: Some(20),
    name: "GPIO20",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::PCM_DIN, // ALT0
        // TODO ALT1: SMI SD12
        // TODO ALT2: DPI D16
        // TODO ALT3: I2CSL MOSI
        // TODO PinFunction::SPI1_MOSI, // ALT4
        // TODO ALT5: GPCLK0
    ],
};

pub const PIN_39: PinDescription = PinDescription {
    board_pin_number: 39,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
};

pub const PIN_40: PinDescription = PinDescription {
    board_pin_number: 40,
    bcm_pin_number: Some(21),
    name: "GPIO21",
    options: &[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO PinFunction::PCM_DOUT,  // ALT0
        // TODO ALT1: SMI SD13
        // TODO ALT2: DPI D17
        // TODO ALT3: I2CSL CE
        // TODO PinFunction::SPI1_SCLK, // ALT4
        // TODO ALT5: GPCLK1
    ],
};
