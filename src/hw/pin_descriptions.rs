use crate::hw::pin_description::PinDescription;
use crate::hw::{InputPull, PinFunction};
use std::borrow::Cow;

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

pub(crate) const PIN_1: PinDescription = PinDescription {
    bpn: 1,
    bcm: None,
    name: Cow::Borrowed("3V3"),
    options: Cow::Borrowed(&[PinFunction::Power3V3]),
};

pub(crate) const PIN_2: PinDescription = PinDescription {
    bpn: 2,
    bcm: None,
    name: Cow::Borrowed("5V"),
    options: Cow::Borrowed(&[PinFunction::Power5V]),
};

/// "Pins GPIO2 and GPIO3 have fixed pull-up resistors"
pub(crate) const PIN_3: PinDescription = PinDescription {
    bpn: 3,
    bcm: Some(2),
    name: Cow::Borrowed("GPIO2"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO ALT0::I2C1_SDA / SDA1
        // TODO ALT1: SMI SA3
        // TODO ALT2: DPI VSYNC / LCD_VSYNC
        // TODO ALT3: SPI3_MOSI / AVEOUT_VSYNC
        // TODO ALT4: CTS2 / AVEIN_VSYNC
        // TODO ALT5: I2C3_SDA / SDA3
    ]),
};

pub(crate) const PIN_4: PinDescription = PinDescription {
    bpn: 4,
    bcm: None,
    name: Cow::Borrowed("5V"),
    options: Cow::Borrowed(&[PinFunction::Power5V]),
};

/// "Pins GPIO2 and GPIO3 have fixed pull-up resistors"
pub(crate) const PIN_5: PinDescription = PinDescription {
    bpn: 5,
    bcm: Some(3),
    name: Cow::Borrowed("GPIO3"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO ALT0::I2C1_SCL / SCL1
        // TODO ALT1: SMI SA2
        // TODO ALT2: DPI_HSYNC / LCD_HSYNC
        // TODO ALT3: SPI3_SCLK / AVEOUT_HSYNC
        // TODO ALT4: RTS2 / AVEIN_HSYNC
        // TODO ALT5: I2C3_SCL / SCL3
    ]),
};

pub(crate) const PIN_6: PinDescription = PinDescription {
    bpn: 6,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[PinFunction::Ground]),
};

pub(crate) const PIN_7: PinDescription = PinDescription {
    bpn: 7,
    bcm: Some(4),
    name: Cow::Borrowed("GPIO4"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO ALT0::GPCLK0,
        // TODO ALT1: SMI SA1 / SA1
        // TODO ALT2: DPI_D0
        // TODO ALT3: SPI4_CE0_N / AVEOUT_VID0
        // TODO ALT4: TXD3 / AVEIN_VID0
        // TODO ALT5: SDA3 / JTAG_TDI
    ]),
};

pub(crate) const PIN_8: PinDescription = PinDescription {
    bpn: 8,
    bcm: Some(14),
    name: Cow::Borrowed("GPIO14"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: UART0_TXD / TXD0
        // TODO ALT1: SMI SD6 / SD6
        // TODO ALT2: DSI_D10
        // TODO ALT3: SPI5_MOSI / AVEOUT_VID10
        // TODO ALT4: CTS5 / AVEIN_VID10
        // TODO ALT5: TXD1 / UART1_TXD
    ]),
};

pub(crate) const PIN_9: PinDescription = PinDescription {
    bpn: 9,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[PinFunction::Ground]),
};

pub(crate) const PIN_10: PinDescription = PinDescription {
    bpn: 10,
    bcm: Some(15),
    name: Cow::Borrowed("GPIO15"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: UART0_RXD / RXD0
        // TODO ALT1: SMI SD7
        // TODO ALT2: DPI_D11
        // TODO ALT3: SPI5_SCLK / AVEOUT VID11
        // TODO ALT4: RTS5 / AVEIN VID11
        // TODO ALT5: RXD1 / UART1_RXD
    ]),
};

pub(crate) const PIN_11: PinDescription = PinDescription {
    bpn: 11,
    bcm: Some(17),
    name: Cow::Borrowed("GPIO17"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: <reserved>
        // TODO ALT1: SMI SD9
        // TODO ALT2: DPI_D13
        // TODO ALT3: UART0 RTS / RTS0
        // TODO ALT4: SPI1_CE1_N
        // TODO ALT5: UART1 RTS / RTS1
    ]),
};

pub(crate) const PIN_12: PinDescription = PinDescription {
    bpn: 12,
    bcm: Some(18),
    name: Cow::Borrowed("GPIO18"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: PCM_CLK
        // TODO ALT1: SMI SD10
        // TODO ALT2: DPI_D14
        // TODO ALT3: SPI6_CE0_N
        // TODO ALT4: SPI1_CE0_N
        // TODO ALT5: PWM0_0
    ]),
};

pub(crate) const PIN_13: PinDescription = PinDescription {
    bpn: 13,
    bcm: Some(27),
    name: Cow::Borrowed("GPIO27"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: SD0_DAT3
        // TODO ALT1: <reserved>
        // TODO ALT2: DPI_D23
        // TODO ALT3: SD1_DAT3
        // TODO ALT4: ARM_TMS / JTA TMS
        // TODO ALT5:
    ]),
};

pub(crate) const PIN_14: PinDescription = PinDescription {
    bpn: 14,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[PinFunction::Ground]),
};

pub(crate) const PIN_15: PinDescription = PinDescription {
    bpn: 15,
    bcm: Some(22),
    name: Cow::Borrowed("GPIO22"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: SD0_CLK
        // TODO ALT1: SMI SD14 / SD14
        // TODO ALT2: DPI_D18
        // TODO ALT3: SD1_CLK
        // TODO ALT4: ARM_TRST / JTA TRST
        // TODO ALT5: SDA6
    ]),
};

pub(crate) const PIN_16: PinDescription = PinDescription {
    bpn: 16,
    bcm: Some(23),
    name: Cow::Borrowed("GPIO23"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: SD0 CMD
        // TODO ALT1: SMI SD15 / SD15
        // TODO ALT2: DPI_D19
        // TODO ALT3: SD1_CMD
        // TODO ALT4: ARM_RTCK / JTA RTCK
        // TODO ALT5: SCL6
    ]),
};

pub(crate) const PIN_17: PinDescription = PinDescription {
    bpn: 17,
    bcm: None,
    name: Cow::Borrowed("3V3"),
    options: Cow::Borrowed(&[PinFunction::Power3V3]),
};

pub(crate) const PIN_18: PinDescription = PinDescription {
    bpn: 18,
    bcm: Some(24),
    name: Cow::Borrowed("GPIO24"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: SD0_DAT0
        // TODO ALT1: SMI SD16 / SD16
        // TODO ALT2: DPI_D20
        // TODO ALT3: SD1_DAT0
        // TODO ALT4: ARM_TDO / JTA TDO
        // TODO ALT5: SPI3_CE1_N
    ]),
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
pub(crate) const PIN_19: PinDescription = PinDescription {
    bpn: 19,
    bcm: Some(10),
    name: Cow::Borrowed("GPIO10"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: SPI0_MOSI
        // TODO ALT1: SMI SD2
        // TODO ALT2: DPI_D6
        // TODO ALT3: BSCSL SDA / MOSI / AVEOUT VID6
        // TODO ALT4: CTS4 / AVEIN VID6
        // TODO ALT5: SDA5
    ]),
};

pub(crate) const PIN_20: PinDescription = PinDescription {
    bpn: 20,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[PinFunction::Ground]),
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
pub(crate) const PIN_21: PinDescription = PinDescription {
    bpn: 21,
    bcm: Some(9),
    name: Cow::Borrowed("GPIO9"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0:SPI0_MISO
        // TODO ALT1: SMI SD1
        // TODO ALT2: DPI_D5
        // TODO ALT3: BSCSL / MISO / AVEOUT VID5
        // TODO ALT4: RXD4 / AVEIN VID5
        // TODO ALT5: SCL4
    ]),
};

pub(crate) const PIN_22: PinDescription = PinDescription {
    bpn: 22,
    bcm: Some(25),
    name: Cow::Borrowed("GPIO25"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: SD0_DAT1
        // TODO ALT1: SMI_SD17 / SD17
        // TODO ALT2: DPI_D21
        // TODO ALT3: SD1_DAT1
        // TODO ALT4: ARM_TCK / JTAG TCK
        // TODO ALT5: SPI4_CE1_N
    ]),
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
pub(crate) const PIN_23: PinDescription = PinDescription {
    bpn: 23,
    bcm: Some(11),
    name: Cow::Borrowed("GPIO11"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: SPI0_SCLK
        // TODO ALT1: SMI SD3
        // TODO ALT2: DPI_D7
        // TODO ALT3: BSCSL SCL / SCLK / AVEOUT VID7
        // TODO ALT4: RTS4 / AVEIN VID7
        // TODO ALT5: SCL5
    ]),
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
pub(crate) const PIN_24: PinDescription = PinDescription {
    bpn: 24,
    bcm: Some(8),
    name: Cow::Borrowed("GPIO8"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO ALT0: SPI0_CE0_N
        // TODO ALT1: SMI SD0 / SD0
        // TODO ALT2: DPI_D4
        // TODO ALT3: BSCSL / CE_N / AVEOUT VID4
        // TODO ALT4: TXD4 / AVEIN VID4
        // TODO ALT5: SDA4
    ]),
};

pub(crate) const PIN_25: PinDescription = PinDescription {
    bpn: 25,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[PinFunction::Ground]),
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
pub(crate) const PIN_26: PinDescription = PinDescription {
    bpn: 26,
    bcm: Some(7),
    name: Cow::Borrowed("GPIO7"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO ALT0: SPI0_CE1_N
        // TODO ALT1: SMI SWE_N / SRW_N
        // TODO ALT2: DPI_D3
        // TODO ALT3: SPI4_SCLK / AVEOUT VID3
        // TODO ALT4: RTS3 / AVEIN VID3
        // TODO ALT5: SCL4
    ]),
};

pub(crate) const PIN_27: PinDescription = PinDescription {
    bpn: 27,
    bcm: None,
    name: Cow::Borrowed("GPIO0"), // EEPROM ID_SD for HAT identification
    options: Cow::Borrowed(&[
        // TODO ALT0: I2C0 SDA / SDA0
        // TODO ALT1: SMI SA5
        // TODO ALT2: DPI CLK / PCLK
        // TODO ALT3: SPI3_CE0_N / AVEOUT VCLK
        // TODO ALT4: TXD2 / AVEIN VCLK
        // TODO ALT5: SDA6
    ]),
};

pub(crate) const PIN_28: PinDescription = PinDescription {
    bpn: 28,
    bcm: None,
    name: Cow::Borrowed("GPIO1"), // EEPROM ID_SCL for HAT identification
    options: Cow::Borrowed(&[
        // TODO ALT0: I2C0 SDL / SCL0
        // TODO ALT1: SMI SA4
        // TODO ALT2: DPI DEN / DE
        // TODO ALT3: SPI3_MISO / AVEOUT DSYNC?
        // TODO ALT4: RXD2 / AVEIN DSYNC?
        // TODO ALT5: SCL6
    ]),
};

pub(crate) const PIN_29: PinDescription = PinDescription {
    bpn: 29,
    bcm: Some(5),
    name: Cow::Borrowed("GPIO5"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO ALT0::GPCLK1
        // TODO ALT1: SMI_SA0 / SA0
        // TODO ALT2: DPI_D1
        // TODO ALT3: SPI4_MISO / AVEOUT VID1
        // TODO ALT4: RXD3 / AVEIN VID1
        // TODO ALT5: SCL3 / JTA TDO
    ]),
};

pub(crate) const PIN_30: PinDescription = PinDescription {
    bpn: 30,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[PinFunction::Ground]),
};

pub(crate) const PIN_31: PinDescription = PinDescription {
    bpn: 31,
    bcm: Some(6),
    name: Cow::Borrowed("GPIO6"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // TODO ALT0::GPCLK2
        // TODO ALT1: SMI SOE_N / SOE_N / SE
        // TODO ALT2: DPI_D2
        // TODO ALT3: SPI4_MOSI / AVEOUT VID2
        // TODO ALT4: CTS3 / AVEIN VID2
        // TODO ALT5: SDA4 / JTA RTCK
    ]),
};

pub(crate) const PIN_32: PinDescription = PinDescription {
    bpn: 32,
    bcm: Some(12),
    name: Cow::Borrowed("GPIO12"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: PWM0 / PWM0_0
        // TODO ALT1: SMI SD4
        // TODO ALT2: DPI_D8
        // TODO ALT3: SPI5_CE0_N / AVEOUT VID8
        // TODO ALT4: TXD5 / AVEIN VID8
        // TODO ALT5: SDA5 / JTA TMS
    ]),
};

pub(crate) const PIN_33: PinDescription = PinDescription {
    bpn: 33,
    bcm: Some(13),
    name: Cow::Borrowed("GPIO13"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: PWM1 / PWM0_1
        // TODO ALT1: SMI SD5
        // TODO ALT2: DPI_D9
        // TODO ALT3: SPI5_MISO / AVEOUT VID9
        // TODO ALT4: RXD5 / AVEIN VID9
        // TODO ALT5: SCL5 / JTA TCK
    ]),
};

pub(crate) const PIN_34: PinDescription = PinDescription {
    bpn: 34,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[PinFunction::Ground]),
};

pub(crate) const PIN_35: PinDescription = PinDescription {
    bpn: 35,
    bcm: Some(19),
    name: Cow::Borrowed("GPIO19"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: PCM_FS
        // TODO ALT1: SMI SD11
        // TODO ALT2: DPI_D15
        // TODO ALT3: SPI6_MISO
        // TODO ALT4: SPI1_MISO
        // TODO ALT5: PWM0_1
    ]),
};

pub(crate) const PIN_36: PinDescription = PinDescription {
    bpn: 36,
    bcm: Some(16),
    name: Cow::Borrowed("GPIO16"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: <reserved>
        // TODO ALT1: SMI SD8
        // TODO ALT2: DPI_D12
        // TODO ALT3: UART0 CTS / CTS0
        // TODO ALT4: SPI1_CE2_N
        // TODO ALT5: UART1 CTS
    ]),
};

pub(crate) const PIN_37: PinDescription = PinDescription {
    bpn: 37,
    bcm: Some(26),
    name: Cow::Borrowed("GPIO26"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: SD0_DAT2
        // TODO ALT1: <reserved>>
        // TODO ALT2: DPI_D22
        // TODO ALT3: SD1_DAT2
        // TODO ALT4: ARM_TDI / JTA TDI
        // TODO ALT5: SPI5_CE1_N
    ]),
};

pub(crate) const PIN_38: PinDescription = PinDescription {
    bpn: 38,
    bcm: Some(20),
    name: Cow::Borrowed("GPIO20"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: PCM_DIN
        // TODO ALT1: SMI SD12
        // TODO ALT2: DPI_D16
        // TODO ALT3: SPI6_MOSI
        // TODO ALT4: SPI1_MOSI
        // TODO ALT5: GPCLK0
    ]),
};

pub(crate) const PIN_39: PinDescription = PinDescription {
    bpn: 39,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[PinFunction::Ground]),
};

pub(crate) const PIN_40: PinDescription = PinDescription {
    bpn: 40,
    bcm: Some(21),
    name: Cow::Borrowed("GPIO21"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // TODO ALT0: PCM_DOUT
        // TODO ALT1: SMI SD13
        // TODO ALT2: DPI_D17
        // TODO ALT3: SPI6_SCLK
        // TODO ALT4: SPI1_SCLK
        // TODO ALT5: GPCLK1
    ]),
};
