use crate::hw_definition::config::InputPull;
use crate::hw_definition::PinLevel;
use serde::{Deserialize, Serialize};

/// For SPI interfaces see [here](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
///
/// Standard mode
/// In Standard SPI mode the peripheral implements the standard three-wire serial protocol
/// * SCLK - serial clock
/// * CE   - chip enable (often called chip select)
/// * MOSI - master out slave in
/// * MISO - master in slave out
///
/// Bidirectional mode
/// In bidirectional SPI mode the same SPI standard is implemented, except that a single wire
/// is used for data (MOMI) instead of the two used in standard mode (MISO and MOSI).
/// In this mode, the MOSI pin serves as MOMI pin.
/// * SCLK - serial clock
/// * CE   - chip enable (often called chip select)
/// * MOMI - master out master in
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum PinFunction {
    None,

    /// HAT ID related functions - two pins to talk to HAT EEPROM via I2C
    I2C_EEPROM_ID_SD,
    I2C_EEPROM_ID_SC,

    /// Power and Ground functions
    Power3V3,
    Power5V,
    Ground,

    /// GPIO functions
    Input(Option<InputPull>),
    Output(Option<PinLevel>),
    /*
    /// General Purpose Clock functions (from https://pinout.xyz/pinout/gpclk)
    GPCLK0,
    GPCLK1,
    GPCLK2,

    /// I2C bus functions
    I2C1_SDA,
    I2C1_SCL,
    I2C3_SDA,
    I2C3_SCL,
    I2C4_SDA,
    I2C4_SCL,
    I2C5_SDA,
    I2C5_SCL,
    I2C6_SDA,
    I2C6_SCL,

    /// SPI Interface #0
    SPI0_MOSI,
    /// Bi-directional mode
    SPI0_MOMI,
    SPI0_MISO,
    SPI0_SCLK,
    SPI0_CE0_N,
    SPI0_CE1_N,

    // SPI Interface #0
    SPI1_MOSI,
    /// Bi-directional mode
    SPI1_MOMI,
    SPI1_MISO,
    SPI1_SCLK,
    SPI1_CE0_N,
    SPI1_CE1_N,
    SPI1_CE2_N,

    /// PWM functions - two pins each use these
    PWM0,
    PWM1,

    /// UART functions
    /// UART0 - Transmit
    UART0_TXD,
    /// UART0 - Receive
    UART0_RXD,

    /// PCM functions - how uncompressed digital audio is encoded
    PCM_FS,
    /// PCM Data In
    PCM_DIN,
    /// PCM Data Out
    PCM_DOUT,
    /// PCM CLock
    PCM_CLK,
     */
}
