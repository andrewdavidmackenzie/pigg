use serde::{Deserialize, Serialize};

pub type PinLevel = bool;

#[derive(Debug)]
#[allow(dead_code)]
pub struct GPIOState {
    pub pin_state: [Option<PinLevel>; 40] // TODO make private later
}

// All the possible functions a pin can be given
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum PinFunction {
    Power3V3,
    Power5V,
    Ground,
    Input,
    Output,
    SDA1,
    I2C,
    SCL1,
    SPIO_MOSI,
    SPIO_MISO,
    SPIO_SCLK,
    ID_SD,
    ID,
    EEPROM,
    UART0_TXD,
    UART0_RXD,
    PCM_CLK,
    SPIO_CE0_N,
    SPIO_CE1_N,
    ID_SC
}

// [board_pin_number] refer to the pins by the number of the pin printed on the board
// [bcm_pin_number] refer to the pins by the "Broadcom SOC channel" number,
// these are the numbers after "GPIO"
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // TODO remove later
pub struct PinConfig<'a> {
    pub board_pin_number: u8,
    bcm_pin_number: Option<u8>,
    name: &'a str,
    options: &'a[PinFunction], // The set of functions the pin can have, chosen by user config
    config: Option<PinFunction>, // The currently selected function for the pin, if any selected
}

// Model the 40 pin GPIO connections - including Ground, 3.3V and 5V outputs
// If no specific config is set on a pin, it will have None
// I have made array of 40, to the index is "of by one" compared to the [board_pin_number]
// is not used
#[derive(Debug, Clone)]
pub struct GPIOConfig<'a> {
    pub pins: [PinConfig<'a>; 40], // TODO make private later
}

const PIN_1: PinConfig = PinConfig {
    board_pin_number: 1,
    bcm_pin_number: None,
    name: "3V3",
    options: &[PinFunction::Power3V3],
    config: None,
};

const PIN_2: PinConfig = PinConfig {
    board_pin_number: 2,
    bcm_pin_number: None,
    name: "5V",
    options: &[PinFunction::Power5V],
    config: None,
};

const PIN_3: PinConfig = PinConfig {
    board_pin_number: 3,
    bcm_pin_number: Some(2),
    name: "GPIO2",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SDA1, PinFunction::I2C],
    config: None,
};

const PIN_4: PinConfig = PinConfig {
    board_pin_number: 4,
    bcm_pin_number: None,
    name: "5V",
    options: &[PinFunction::Power5V],
    config: None,
};

const PIN_5: PinConfig = PinConfig {
    board_pin_number: 2,
    bcm_pin_number: Some(3),
    name: "GPIO3",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SCL1, PinFunction::I2C],
    config: None,
};

const PIN_6: PinConfig = PinConfig {
    board_pin_number: 6,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
    config: None,
};

const PIN_7: PinConfig = PinConfig {
    board_pin_number: 7,
    bcm_pin_number: Some(4),
    name: "GPIO4",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_8: PinConfig = PinConfig {
    board_pin_number: 8,
    bcm_pin_number: Some(14),
    name: "GPIO14",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::UART0_TXD],
    config: None,
};

const PIN_9: PinConfig = PinConfig {
    board_pin_number: 9,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
    config: None,
};

const PIN_10: PinConfig = PinConfig {
    board_pin_number: 10,
    bcm_pin_number: Some(15),
    name: "GPIO15",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::UART0_RXD],
    config: None,
};

const PIN_11: PinConfig = PinConfig {
    board_pin_number: 11,
    bcm_pin_number: Some(17),
    name: "GPIO17",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_12: PinConfig = PinConfig {
    board_pin_number: 12,
    bcm_pin_number: Some(18),
    name: "GPIO18",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::PCM_CLK],
    config: None,
};

const PIN_13: PinConfig = PinConfig {
    board_pin_number: 13,
    bcm_pin_number: Some(27),
    name: "GPIO27",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_14: PinConfig = PinConfig {
    board_pin_number: 14,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
    config: None,
};

const PIN_15: PinConfig = PinConfig {
    board_pin_number: 15,
    bcm_pin_number: Some(22),
    name: "GPIO22",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_16: PinConfig = PinConfig {
    board_pin_number: 16,
    bcm_pin_number: Some(23),
    name: "GPIO23",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_17: PinConfig = PinConfig {
    board_pin_number: 17,
    bcm_pin_number: None,
    name: "3V3",
    options: &[PinFunction::Power3V3],
    config: None,
};

const PIN_18: PinConfig = PinConfig {
    board_pin_number: 18,
    bcm_pin_number: Some(24),
    name: "GPIO24",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_19: PinConfig = PinConfig {
    board_pin_number: 19,
    bcm_pin_number: Some(10),
    name: "GPIO10",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SPIO_MOSI],
    config: None,
};

const PIN_20: PinConfig = PinConfig {
    board_pin_number: 20,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
    config: None,
};

const PIN_21: PinConfig = PinConfig {
    board_pin_number: 21,
    bcm_pin_number: Some(9),
    name: "GPIO9",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SPIO_MISO],
    config: None,
};

const PIN_22: PinConfig = PinConfig {
    board_pin_number: 22,
    bcm_pin_number: Some(25),
    name: "GPIO25",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_23: PinConfig = PinConfig {
    board_pin_number: 23,
    bcm_pin_number: Some(11),
    name: "GPIO11",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SPIO_SCLK],
    config: None,
};

const PIN_24: PinConfig = PinConfig {
    board_pin_number: 24,
    bcm_pin_number: Some(8),
    name: "GPIO8",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SPIO_CE0_N],
    config: None,
};

const PIN_25: PinConfig = PinConfig {
    board_pin_number: 25,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
    config: None,
};

const PIN_26: PinConfig = PinConfig {
    board_pin_number: 26,
    bcm_pin_number: Some(7),
    name: "GPIO7",
    options: &[PinFunction::Input, PinFunction::Output, PinFunction::SPIO_CE1_N],
    config: None,
};

const PIN_27: PinConfig = PinConfig {
    board_pin_number: 27,
    bcm_pin_number: None,
    name: "ID_SD",
    options: &[PinFunction::ID_SD, PinFunction::I2C, PinFunction::ID, PinFunction::EEPROM],
    config: None,
};

const PIN_28: PinConfig = PinConfig {
    board_pin_number: 28,
    bcm_pin_number: None,
    name: "ID_SC",
    options: &[PinFunction::ID_SC, PinFunction::I2C, PinFunction::ID, PinFunction::EEPROM],
    config: None,
};

const PIN_29: PinConfig = PinConfig {
    board_pin_number: 29,
    bcm_pin_number: Some(5),
    name: "GPIO5",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_30: PinConfig = PinConfig {
    board_pin_number: 30,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
    config: None,
};

const PIN_31: PinConfig = PinConfig {
    board_pin_number: 31,
    bcm_pin_number: Some(6),
    name: "GPIO6",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_32: PinConfig = PinConfig {
    board_pin_number: 32,
    bcm_pin_number: Some(12),
    name: "GPIO12",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_33: PinConfig = PinConfig {
    board_pin_number: 33,
    bcm_pin_number: Some(13),
    name: "GPIO13",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_34: PinConfig = PinConfig {
    board_pin_number: 34,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
    config: None,
};

const PIN_35: PinConfig = PinConfig {
    board_pin_number: 35,
    bcm_pin_number: Some(19),
    name: "GPIO19",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_36: PinConfig = PinConfig {
    board_pin_number: 36,
    bcm_pin_number: Some(16),
    name: "GPIO16",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_37: PinConfig = PinConfig {
    board_pin_number: 37,
    bcm_pin_number: Some(26),
    name: "GPIO26",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_38: PinConfig = PinConfig {
    board_pin_number: 38,
    bcm_pin_number: Some(20),
    name: "GPIO20",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

const PIN_39: PinConfig = PinConfig {
    board_pin_number: 39,
    bcm_pin_number: None,
    name: "Ground",
    options: &[PinFunction::Ground],
    config: None,
};

const PIN_40: PinConfig = PinConfig {
    board_pin_number: 40,
    bcm_pin_number: Some(21),
    name: "GPIO21",
    options: &[PinFunction::Input, PinFunction::Output],
    config: None,
};

// Valid for Pi Model B+, Pi 2B, Pi Zero, Pi 3B, and Pi 4B:
impl<'a> Default for GPIOConfig<'a> {
    fn default() -> Self {
        GPIOConfig {
            pins: [PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10,
                   PIN_11, PIN_12, PIN_13, PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20,
                   PIN_21, PIN_22, PIN_23, PIN_24, PIN_25, PIN_26, PIN_27, PIN_28, PIN_29, PIN_30,
                   PIN_31, PIN_32, PIN_33, PIN_34, PIN_35, PIN_36, PIN_37, PIN_38, PIN_39, PIN_40],
        }
    }
}

#[cfg(test)]
mod test {
    use crate::gpio;

    #[test]
    fn create_a_config() {
        let config = gpio::GPIOConfig::new();
        assert_eq!(config.pins[1].config, None);
    }
}