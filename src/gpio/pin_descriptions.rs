use crate::gpio::{PinDescription, PinFunction};

// Valid Pin descriptions for Pi Model B+, Pi 2B, Pi Zero, Pi 3B, and Pi 4B:

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

pub const PIN_3: PinDescription = PinDescription {
    board_pin_number: 3,
    bcm_pin_number: Some(2),
    name: "GPIO2",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SDA1, PinFunction::I2C],
};

pub const PIN_4: PinDescription = PinDescription {
    board_pin_number: 4,
    bcm_pin_number: None,
    name: "5V",
    options: &[PinFunction::Power5V],
};

pub const PIN_5: PinDescription = PinDescription {
    board_pin_number: 5,
    bcm_pin_number: Some(3),
    name: "GPIO3",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SCL1, PinFunction::I2C],
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
    options: &[PinFunction::Input, PinFunction::Output],
};

pub const PIN_8: PinDescription = PinDescription {
    board_pin_number: 8,
    bcm_pin_number: Some(14),
    name: "GPIO14",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::UART0_TXD],
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
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::UART0_RXD],
};

pub const PIN_11: PinDescription = PinDescription {
    board_pin_number: 11,
    bcm_pin_number: Some(17),
    name: "GPIO17",
    options: &[PinFunction::Input, PinFunction::Output],
};

pub const PIN_12: PinDescription = PinDescription {
    board_pin_number: 12,
    bcm_pin_number: Some(18),
    name: "GPIO18",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::PCM_CLK],
};

pub const PIN_13: PinDescription = PinDescription {
    board_pin_number: 13,
    bcm_pin_number: Some(27),
    name: "GPIO27",
    options: &[PinFunction::Input, PinFunction::Output],
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
    options: &[PinFunction::Input, PinFunction::Output],
};

pub const PIN_16: PinDescription = PinDescription {
    board_pin_number: 16,
    bcm_pin_number: Some(23),
    name: "GPIO23",
    options: &[PinFunction::Input, PinFunction::Output],
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
    options: &[PinFunction::Input, PinFunction::Output],
};

pub const PIN_19: PinDescription = PinDescription {
    board_pin_number: 19,
    bcm_pin_number: Some(10),
    name: "GPIO10",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SPIO_MOSI],
};

pub const PIN_20: PinDescription = PinDescription {
    board_pin_number: 20,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
};

pub const PIN_21: PinDescription = PinDescription {
    board_pin_number: 21,
    bcm_pin_number: Some(9),
    name: "GPIO9",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SPIO_MISO],
};

pub const PIN_22: PinDescription = PinDescription {
    board_pin_number: 22,
    bcm_pin_number: Some(25),
    name: "GPIO25",
    options: &[PinFunction::Input, PinFunction::Output],
};

pub const PIN_23: PinDescription = PinDescription {
    board_pin_number: 23,
    bcm_pin_number: Some(11),
    name: "GPIO11",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SPIO_SCLK],
};

pub const PIN_24: PinDescription = PinDescription {
    board_pin_number: 24,
    bcm_pin_number: Some(8),
    name: "GPIO8",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SPIO_CE0_N],
};

pub const PIN_25: PinDescription = PinDescription {
    board_pin_number: 25,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
};

pub const PIN_26: PinDescription = PinDescription {
    board_pin_number: 26,
    bcm_pin_number: Some(7),
    name: "GPIO7",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SPIO_CE1_N],
};

pub const PIN_27: PinDescription = PinDescription {
    board_pin_number: 27,
    bcm_pin_number: None,
    name: "ID_SD",
    options: &[PinFunction::ID_SD, PinFunction::I2C, PinFunction::ID, PinFunction::EEPROM],
};

pub const PIN_28: PinDescription = PinDescription {
    board_pin_number: 28,
    bcm_pin_number: None,
    name: "ID_SC",
    options: &[PinFunction::ID_SC, PinFunction::I2C, PinFunction::ID, PinFunction::EEPROM],
};

pub const PIN_29: PinDescription = PinDescription {
    board_pin_number: 29,
    bcm_pin_number: Some(5),
    name: "GPIO5",
    options: &[PinFunction::Input, PinFunction::Output],
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
    options: &[PinFunction::Input, PinFunction::Output],
};

pub const PIN_32: PinDescription = PinDescription {
    board_pin_number: 32,
    bcm_pin_number: Some(12),
    name: "GPIO12",
    options: &[PinFunction::Input, PinFunction::Output],
};

pub const PIN_33: PinDescription = PinDescription {
    board_pin_number: 33,
    bcm_pin_number: Some(13),
    name: "GPIO13",
    options: &[PinFunction::Input, PinFunction::Output],
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
    options: &[PinFunction::Input, PinFunction::Output],
};

pub const PIN_36: PinDescription = PinDescription {
    board_pin_number: 36,
    bcm_pin_number: Some(16),
    name: "GPIO16",
    options: &[PinFunction::Input, PinFunction::Output],
};

pub const PIN_37: PinDescription = PinDescription {
    board_pin_number: 37,
    bcm_pin_number: Some(26),
    name: "GPIO26",
    options: &[PinFunction::Input, PinFunction::Output],
};

pub const PIN_38: PinDescription = PinDescription {
    board_pin_number: 38,
    bcm_pin_number: Some(20),
    name: "GPIO20",
    options: &[PinFunction::Input, PinFunction::Output],
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
    options: &[PinFunction::Input, PinFunction::Output],
};