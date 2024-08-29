use crate::hw_definition::config::InputPull;
use crate::hw_definition::description::PinDescription;
use crate::hw_definition::pin_function::PinFunction;

const PIN_1: PinDescription = PinDescription {
    bpn: 1,
    bcm: Some(0),
    name: "GP0",
    options: &[
        PinFunction::Output(None),
        // SPI0 RX
        // I2C0 SDA
        // UART0 TX
    ],
};

const PIN_2: PinDescription = PinDescription {
    bpn: 2,
    bcm: Some(1),
    name: "GP1",
    options: &[
        PinFunction::Output(None),
        // SPI0 SCL
        // I2C0 SCL
        // UART0 RX
    ],
};

const PIN_3: PinDescription = PinDescription {
    bpn: 3,
    bcm: None,
    name: "Ground",
    options: &[],
};

const PIN_4: PinDescription = PinDescription {
    bpn: 4,
    bcm: Some(2),
    name: "GP2",
    options: &[
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI0 SCK
        // I2C1 SDA
    ],
};

const PIN_5: PinDescription = PinDescription {
    bpn: 5,
    bcm: Some(3),
    name: "GP3",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI0 TX
        // I2C0 SCL
    ],
};

const PIN_6: PinDescription = PinDescription {
    bpn: 6,
    bcm: Some(4),
    name: "GP4",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI0 RX
        // I2C0 SDA
        // UART1 TX
    ],
};

const PIN_7: PinDescription = PinDescription {
    bpn: 7,
    bcm: Some(5),
    name: "GP5",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI0 CSn
        // I2C0 SCL
        // UART1 RX
    ],
};

const PIN_8: PinDescription = PinDescription {
    bpn: 8,
    bcm: None,
    name: "Ground",
    options: &[],
};

const PIN_9: PinDescription = PinDescription {
    bpn: 9,
    bcm: Some(6),
    name: "GP6",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI0 SCK
        // I2C1 SDA
    ],
};

const PIN_10: PinDescription = PinDescription {
    bpn: 10,
    bcm: Some(7),
    name: "GP7",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI0 TX
        // I2C1 SCL
    ],
};

const PIN_11: PinDescription = PinDescription {
    bpn: 11,
    bcm: Some(8),
    name: "GP8",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI1 RX
        // I2C0 SDA
        // UART1 TX
    ],
};

const PIN_12: PinDescription = PinDescription {
    bpn: 12,
    bcm: Some(9),
    name: "GP9",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI1 CSn
        // I2C0 SCL
        // UART1 RX
    ],
};

const PIN_13: PinDescription = PinDescription {
    bpn: 13,
    bcm: None,
    name: "Ground",
    options: &[],
};

const PIN_14: PinDescription = PinDescription {
    bpn: 14,
    bcm: Some(10),
    name: "GP10",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI1 SCK
        // I2C1 SDA
    ],
};

const PIN_15: PinDescription = PinDescription {
    bpn: 15,
    bcm: Some(11),
    name: "GP11",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI1 TX
        // I2C1 SCL
    ],
};

const PIN_16: PinDescription = PinDescription {
    bpn: 16,
    bcm: Some(12),
    name: "GP12",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI1 RX
        // I2C0 SDA
        // UART0 TX
    ],
};

const PIN_17: PinDescription = PinDescription {
    bpn: 17,
    bcm: Some(13),
    name: "GP13",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI1 CSn
        // I2C0 SCL
        // UART0 RX
    ],
};

const PIN_18: PinDescription = PinDescription {
    bpn: 18,
    bcm: None,
    name: "Ground",
    options: &[],
};

const PIN_19: PinDescription = PinDescription {
    bpn: 19,
    bcm: Some(14),
    name: "GP14",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI1 SCK
        // I2C1 SDA
    ],
};

const PIN_20: PinDescription = PinDescription {
    bpn: 20,
    bcm: Some(15),
    name: "GP15",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI1 TX
        // I2C1 SCL
    ],
};

const PIN_21: PinDescription = PinDescription {
    bpn: 21,
    bcm: Some(16),
    name: "GP16",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI0 RX
        // I2C0 SDA
        // UART0 TX
    ],
};

const PIN_22: PinDescription = PinDescription {
    bpn: 22,
    bcm: Some(17),
    name: "GP17",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI0 CSn
        // I2C0 SCL
        // UART0 RX
    ],
};

const PIN_23: PinDescription = PinDescription {
    bpn: 23,
    bcm: None,
    name: "Ground",
    options: &[],
};

const PIN_24: PinDescription = PinDescription {
    bpn: 24,
    bcm: Some(18),
    name: "GP18",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI0 SCK
        // I2C1 SDA
    ],
};

const PIN_25: PinDescription = PinDescription {
    bpn: 25,
    bcm: Some(19),
    name: "GP19",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI0 TX
        // I2C1 SCL
    ],
};

const PIN_26: PinDescription = PinDescription {
    bpn: 26,
    bcm: Some(20),
    name: "GP20",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI0 SDA
    ],
};

const PIN_27: PinDescription = PinDescription {
    bpn: 27,
    bcm: Some(21),
    name: "GP21",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // SPI0 SCL
    ],
};

const PIN_28: PinDescription = PinDescription {
    bpn: 28,
    bcm: None,
    name: "Ground",
    options: &[],
};

const PIN_29: PinDescription = PinDescription {
    bpn: 29,
    bcm: Some(22),
    name: "GP22",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
    ],
};

const PIN_30: PinDescription = PinDescription {
    bpn: 30,
    bcm: None,
    name: "RUN",
    options: &[],
};

const PIN_31: PinDescription = PinDescription {
    bpn: 31,
    bcm: Some(26),
    name: "GP26",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // ADC0
        // I2C1 SDA
    ],
};

const PIN_32: PinDescription = PinDescription {
    bpn: 32,
    bcm: Some(27),
    name: "GP27",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // ADC1
        // I2C1 SCL
    ],
};

const PIN_33: PinDescription = PinDescription {
    bpn: 33,
    bcm: None,
    name: "3V3",
    options: &[],
};

const PIN_34: PinDescription = PinDescription {
    bpn: 34,
    bcm: Some(28),
    name: "GP28",
    options: &[
        PinFunction::Output(None),
        PinFunction::Input(Some(InputPull::PullUp)),
        // ADC2
    ],
};

const PIN_35: PinDescription = PinDescription {
    bpn: 35,
    bcm: None,
    name: "ADC_VREF",
    options: &[],
};

const PIN_36: PinDescription = PinDescription {
    bpn: 36,
    bcm: None,
    name: "3V3(OUT)",
    options: &[],
};

const PIN_37: PinDescription = PinDescription {
    bpn: 37,
    bcm: None,
    name: "3V3_EN",
    options: &[],
};

const PIN_38: PinDescription = PinDescription {
    bpn: 38,
    bcm: None,
    name: "Ground",
    options: &[],
};

const PIN_39: PinDescription = PinDescription {
    bpn: 39,
    bcm: None,
    name: "VSYS",
    options: &[],
};

const PIN_40: PinDescription = PinDescription {
    bpn: 40,
    bcm: None,
    name: "VBUS",
    options: &[],
};

/// Array of 40 [PinDescription]s of the 40 pins on a Pi Pico W
pub const PIN_DESCRIPTIONS: [PinDescription; 40] = [
    PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13,
    PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25,
    PIN_26, PIN_27, PIN_28, PIN_29, PIN_30, PIN_31, PIN_32, PIN_33, PIN_34, PIN_35, PIN_36, PIN_37,
    PIN_38, PIN_39, PIN_40,
];
