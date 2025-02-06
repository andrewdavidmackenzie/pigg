use crate::hw_definition::config::InputPull;
use crate::hw_definition::description::PinDescription;
use crate::hw_definition::pin_function::PinFunction;
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
/// Pin 1 is just a 3.3V power pin
const PIN_1: PinDescription = PinDescription {
    bpn: 1,
    bcm: None,
    name: Cow::Borrowed("3V3"),
    options: Cow::Borrowed(&[]),
};

const PIN_2: PinDescription = PinDescription {
    bpn: 2,
    bcm: None,
    name: Cow::Borrowed("5V"),
    options: Cow::Borrowed(&[]),
};

/// "Pins GPIO2 and GPIO3 have fixed pull-up resistors"
const PIN_3: PinDescription = PinDescription {
    bpn: 3,
    bcm: Some(2),
    name: Cow::Borrowed("GPIO2"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // ALT0::I2C1_SDA / SDA1
        // ALT1: SMI SA3
        // ALT2: DPI VSYNC / LCD_VSYNC
        // ALT3: SPI3_MOSI / AVEOUT_VSYNC
        // ALT4: CTS2 / AVEIN_VSYNC
        // ALT5: I2C3_SDA / SDA3
    ]),
};

const PIN_4: PinDescription = PinDescription {
    bpn: 4,
    bcm: None,
    name: Cow::Borrowed("5V"),
    options: Cow::Borrowed(&[]),
};

/// "Pins GPIO2 and GPIO3 have fixed pull-up resistors"
const PIN_5: PinDescription = PinDescription {
    bpn: 5,
    bcm: Some(3),
    name: Cow::Borrowed("GPIO3"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // ALT0::I2C1_SCL / SCL1
        // ALT1: SMI SA2
        // ALT2: DPI_HSYNC / LCD_HSYNC
        // ALT3: SPI3_SCLK / AVEOUT_HSYNC
        // ALT4: RTS2 / AVEIN_HSYNC
        // ALT5: I2C3_SCL / SCL3
    ]),
};

const PIN_6: PinDescription = PinDescription {
    bpn: 6,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[]),
};

const PIN_7: PinDescription = PinDescription {
    bpn: 7,
    bcm: Some(4),
    name: Cow::Borrowed("GPIO4"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // ALT0::GPCLK0,
        // ALT1: SMI SA1 / SA1
        // ALT2: DPI_D0
        // ALT3: SPI4_CE0_N / AVEOUT_VID0
        // ALT4: TXD3 / AVEIN_VID0
        // ALT5: SDA3 / JTAG_TDI
    ]),
};

const PIN_8: PinDescription = PinDescription {
    bpn: 8,
    bcm: Some(14),
    name: Cow::Borrowed("GPIO14"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: UART0_TXD / TXD0
        // ALT1: SMI SD6 / SD6
        // ALT2: DSI_D10
        // ALT3: SPI5_MOSI / AVEOUT_VID10
        // ALT4: CTS5 / AVEIN_VID10
        // ALT5: TXD1 / UART1_TXD
    ]),
};

const PIN_9: PinDescription = PinDescription {
    bpn: 9,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[]),
};

const PIN_10: PinDescription = PinDescription {
    bpn: 10,
    bcm: Some(15),
    name: Cow::Borrowed("GPIO15"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: UART0_RXD / RXD0
        // ALT1: SMI SD7
        // ALT2: DPI_D11
        // ALT3: SPI5_SCLK / AVEOUT VID11
        // ALT4: RTS5 / AVEIN VID11
        // ALT5: RXD1 / UART1_RXD
    ]),
};

const PIN_11: PinDescription = PinDescription {
    bpn: 11,
    bcm: Some(17),
    name: Cow::Borrowed("GPIO17"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: <reserved>
        // ALT1: SMI SD9
        // ALT2: DPI_D13
        // ALT3: UART0 RTS / RTS0
        // ALT4: SPI1_CE1_N
        // ALT5: UART1 RTS / RTS1
    ]),
};

const PIN_12: PinDescription = PinDescription {
    bpn: 12,
    bcm: Some(18),
    name: Cow::Borrowed("GPIO18"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: PCM_CLK
        // ALT1: SMI SD10
        // ALT2: DPI_D14
        // ALT3: SPI6_CE0_N
        // ALT4: SPI1_CE0_N
        // ALT5: PWM0_0
    ]),
};

const PIN_13: PinDescription = PinDescription {
    bpn: 13,
    bcm: Some(27),
    name: Cow::Borrowed("GPIO27"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: SD0_DAT3
        // ALT1: <reserved>
        // ALT2: DPI_D23
        // ALT3: SD1_DAT3
        // ALT4: ARM_TMS / JTA TMS
        // ALT5:
    ]),
};

const PIN_14: PinDescription = PinDescription {
    bpn: 14,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[]),
};

const PIN_15: PinDescription = PinDescription {
    bpn: 15,
    bcm: Some(22),
    name: Cow::Borrowed("GPIO22"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: SD0_CLK
        // ALT1: SMI SD14 / SD14
        // ALT2: DPI_D18
        // ALT3: SD1_CLK
        // ALT4: ARM_TRST / JTA TRST
        // ALT5: SDA6
    ]),
};

const PIN_16: PinDescription = PinDescription {
    bpn: 16,
    bcm: Some(23),
    name: Cow::Borrowed("GPIO23"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: SD0 CMD
        // ALT1: SMI SD15 / SD15
        // ALT2: DPI_D19
        // ALT3: SD1_CMD
        // ALT4: ARM_RTCK / JTA RTCK
        // ALT5: SCL6
    ]),
};

const PIN_17: PinDescription = PinDescription {
    bpn: 17,
    bcm: None,
    name: Cow::Borrowed("3V3"),
    options: Cow::Borrowed(&[]),
};

const PIN_18: PinDescription = PinDescription {
    bpn: 18,
    bcm: Some(24),
    name: Cow::Borrowed("GPIO24"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: SD0_DAT0
        // ALT1: SMI SD16 / SD16
        // ALT2: DPI_D20
        // ALT3: SD1_DAT0
        // ALT4: ARM_TDO / JTA TDO
        // ALT5: SPI3_CE1_N
    ]),
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
const PIN_19: PinDescription = PinDescription {
    bpn: 19,
    bcm: Some(10),
    name: Cow::Borrowed("GPIO10"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: SPI0_MOSI
        // ALT1: SMI SD2
        // ALT2: DPI_D6
        // ALT3: BSCSL SDA / MOSI / AVEOUT VID6
        // ALT4: CTS4 / AVEIN VID6
        // ALT5: SDA5
    ]),
};

const PIN_20: PinDescription = PinDescription {
    bpn: 20,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[]),
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
const PIN_21: PinDescription = PinDescription {
    bpn: 21,
    bcm: Some(9),
    name: Cow::Borrowed("GPIO9"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0:SPI0_MISO
        // ALT1: SMI SD1
        // ALT2: DPI_D5
        // ALT3: BSCSL / MISO / AVEOUT VID5
        // ALT4: RXD4 / AVEIN VID5
        // ALT5: SCL4
    ]),
};

const PIN_22: PinDescription = PinDescription {
    bpn: 22,
    bcm: Some(25),
    name: Cow::Borrowed("GPIO25"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: SD0_DAT1
        // ALT1: SMI_SD17 / SD17
        // ALT2: DPI_D21
        // ALT3: SD1_DAT1
        // ALT4: ARM_TCK / JTAG TCK
        // ALT5: SPI4_CE1_N
    ]),
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
const PIN_23: PinDescription = PinDescription {
    bpn: 23,
    bcm: Some(11),
    name: Cow::Borrowed("GPIO11"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: SPI0_SCLK
        // ALT1: SMI SD3
        // ALT2: DPI_D7
        // ALT3: BSCSL SCL / SCLK / AVEOUT VID7
        // ALT4: RTS4 / AVEIN VID7
        // ALT5: SCL5
    ]),
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
const PIN_24: PinDescription = PinDescription {
    bpn: 24,
    bcm: Some(8),
    name: Cow::Borrowed("GPIO8"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // ALT0: SPI0_CE0_N
        // ALT1: SMI SD0 / SD0
        // ALT2: DPI_D4
        // ALT3: BSCSL / CE_N / AVEOUT VID4
        // ALT4: TXD4 / AVEIN VID4
        // ALT5: SDA4
    ]),
};

const PIN_25: PinDescription = PinDescription {
    bpn: 25,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[]),
};

/// See [SPI Interface description](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
const PIN_26: PinDescription = PinDescription {
    bpn: 26,
    bcm: Some(7),
    name: Cow::Borrowed("GPIO7"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // ALT0: SPI0_CE1_N
        // ALT1: SMI SWE_N / SRW_N
        // ALT2: DPI_D3
        // ALT3: SPI4_SCLK / AVEOUT VID3
        // ALT4: RTS3 / AVEIN VID3
        // ALT5: SCL4
    ]),
};

const PIN_27: PinDescription = PinDescription {
    bpn: 27,
    bcm: None,
    name: Cow::Borrowed("GPIO0"), // EEPROM ID_SD for HAT identification
    options: Cow::Borrowed(&[
        // ALT0: I2C0 SDA / SDA0
        // ALT1: SMI SA5
        // ALT2: DPI CLK / PCLK
        // ALT3: SPI3_CE0_N / AVEOUT VCLK
        // ALT4: TXD2 / AVEIN VCLK
        // ALT5: SDA6
    ]),
};

const PIN_28: PinDescription = PinDescription {
    bpn: 28,
    bcm: None,
    name: Cow::Borrowed("GPIO1"), // EEPROM ID_SCL for HAT identification
    options: Cow::Borrowed(&[
        // ALT0: I2C0 SDL / SCL0
        // ALT1: SMI SA4
        // ALT2: DPI DEN / DE
        // ALT3: SPI3_MISO / AVEOUT DSYNC?
        // ALT4: RXD2 / AVEIN DSYNC?
        // ALT5: SCL6
    ]),
};

const PIN_29: PinDescription = PinDescription {
    bpn: 29,
    bcm: Some(5),
    name: Cow::Borrowed("GPIO5"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // ALT0::GPCLK1
        // ALT1: SMI_SA0 / SA0
        // ALT2: DPI_D1
        // ALT3: SPI4_MISO / AVEOUT VID1
        // ALT4: RXD3 / AVEIN VID1
        // ALT5: SCL3 / JTA TDO
    ]),
};

const PIN_30: PinDescription = PinDescription {
    bpn: 30,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[]),
};

const PIN_31: PinDescription = PinDescription {
    bpn: 31,
    bcm: Some(6),
    name: Cow::Borrowed("GPIO6"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullUp)),
        PinFunction::Output(None),
        // ALT0::GPCLK2
        // ALT1: SMI SOE_N / SOE_N / SE
        // ALT2: DPI_D2
        // ALT3: SPI4_MOSI / AVEOUT VID2
        // ALT4: CTS3 / AVEIN VID2
        // ALT5: SDA4 / JTA RTCK
    ]),
};

const PIN_32: PinDescription = PinDescription {
    bpn: 32,
    bcm: Some(12),
    name: Cow::Borrowed("GPIO12"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: PWM0 / PWM0_0
        // ALT1: SMI SD4
        // ALT2: DPI_D8
        // ALT3: SPI5_CE0_N / AVEOUT VID8
        // ALT4: TXD5 / AVEIN VID8
        // ALT5: SDA5 / JTA TMS
    ]),
};

const PIN_33: PinDescription = PinDescription {
    bpn: 33,
    bcm: Some(13),
    name: Cow::Borrowed("GPIO13"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: PWM1 / PWM0_1
        // ALT1: SMI SD5
        // ALT2: DPI_D9
        // ALT3: SPI5_MISO / AVEOUT VID9
        // ALT4: RXD5 / AVEIN VID9
        // ALT5: SCL5 / JTA TCK
    ]),
};

const PIN_34: PinDescription = PinDescription {
    bpn: 34,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[]),
};

const PIN_35: PinDescription = PinDescription {
    bpn: 35,
    bcm: Some(19),
    name: Cow::Borrowed("GPIO19"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: PCM_FS
        // ALT1: SMI SD11
        // ALT2: DPI_D15
        // ALT3: SPI6_MISO
        // ALT4: SPI1_MISO
        // ALT5: PWM0_1
    ]),
};

const PIN_36: PinDescription = PinDescription {
    bpn: 36,
    bcm: Some(16),
    name: Cow::Borrowed("GPIO16"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: <reserved>
        // ALT1: SMI SD8
        // ALT2: DPI_D12
        // ALT3: UART0 CTS / CTS0
        // ALT4: SPI1_CE2_N
        // ALT5: UART1 CTS
    ]),
};

const PIN_37: PinDescription = PinDescription {
    bpn: 37,
    bcm: Some(26),
    name: Cow::Borrowed("GPIO26"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: SD0_DAT2
        // ALT1: <reserved>>
        // ALT2: DPI_D22
        // ALT3: SD1_DAT2
        // ALT4: ARM_TDI / JTA TDI
        // ALT5: SPI5_CE1_N
    ]),
};

const PIN_38: PinDescription = PinDescription {
    bpn: 38,
    bcm: Some(20),
    name: Cow::Borrowed("GPIO20"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: PCM_DIN
        // ALT1: SMI SD12
        // ALT2: DPI_D16
        // ALT3: SPI6_MOSI
        // ALT4: SPI1_MOSI
        // ALT5: GPCLK0
    ]),
};

const PIN_39: PinDescription = PinDescription {
    bpn: 39,
    bcm: None,
    name: Cow::Borrowed("Ground"),
    options: Cow::Borrowed(&[]),
};

const PIN_40: PinDescription = PinDescription {
    bpn: 40,
    bcm: Some(21),
    name: Cow::Borrowed("GPIO21"),
    options: Cow::Borrowed(&[
        PinFunction::Input(Some(InputPull::PullDown)),
        PinFunction::Output(None),
        // ALT0: PCM_DOUT
        // ALT1: SMI SD13
        // ALT2: DPI_D17
        // ALT3: SPI6_SCLK
        // ALT4: SPI1_SCLK
        // ALT5: GPCLK1
    ]),
};

/// Model the 40 pin GPIO connections - including Ground, 3.3V and 5V outputs
/// For now, we will use the same descriptions for all hardware
/// NOTE: They are ordered by rows in the physical layout, so the rows go like this:
///  1   -   2
///  3   -   4
///  5   -   6
/// ...
/// 39   -  40
//noinspection DuplicatedCode
pub(crate) const GPIO_PIN_DESCRIPTIONS: [PinDescription; 40] = [
    PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13,
    PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25,
    PIN_26, PIN_27, PIN_28, PIN_29, PIN_30, PIN_31, PIN_32, PIN_33, PIN_34, PIN_35, PIN_36, PIN_37,
    PIN_38, PIN_39, PIN_40,
];
