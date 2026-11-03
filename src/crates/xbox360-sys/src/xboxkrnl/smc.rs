//! Typed requests and replies of the system management controller (SMC).
//!
//! The encoding follows SMC firmware 510301 (see `xboxkrnl/hal.h`). A
//! query (command below 0x80) gets a reply whose byte 0 repeats the
//! command; reply byte `n + 1` is request data byte `n`. Other commands get
//! no reply. The SMC does not answer a query it does not know, so only the
//! queries of [`SmcRequest`] are sent.

use core::fmt;

use super::hardware::SmcCommand;
use super::hardware::SmcMessage;

/// A request that [`SmcRequest::from_message`] or a reply parser does not
/// accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SmcDecodeError {
    /// The message is not the reply of `expected`: byte 0 is `found`.
    WrongCommand { expected: SmcCommand, found: u8 },
    /// Byte 0 is a command without an [`SmcRequest`].
    UnsupportedCommand(u8),
    /// A fixed fan speed above 100 percent.
    InvalidFanSpeed(u8),
    /// A power LED mode the SMC ignores.
    InvalidPowerLedMode(u8),
}

impl fmt::Display for SmcDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongCommand { expected, found } => write!(
                f,
                "SMC message {found:#04x} is not the reply of {expected:?} ({:#04x})",
                expected.raw()
            ),
            Self::UnsupportedCommand(command) => {
                write!(f, "SMC command {command:#04x} has no typed request")
            }
            Self::InvalidFanSpeed(speed) => {
                write!(
                    f,
                    "fan speed byte {speed:#04x} selects more than 100 percent"
                )
            }
            Self::InvalidPowerLedMode(mode) => {
                write!(f, "power LED mode {mode} is ignored by the SMC")
            }
        }
    }
}

impl core::error::Error for SmcDecodeError {}

/// Check that `reply` answers `command`.
const fn expect_reply(reply: &SmcMessage, command: SmcCommand) -> Result<(), SmcDecodeError> {
    if reply.0[0] == command.raw() {
        Ok(())
    } else {
        Err(SmcDecodeError::WrongCommand {
            expected: command,
            found: reply.0[0],
        })
    }
}

/// A temperature in 1/256 degrees: the 8.8 fixed point value of
/// [`SmcCommand::QueryTemperatures`]. The unit is degrees Celsius
/// [INFERENCE from the kernel's log messages].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Temperature(u16);

impl Temperature {
    /// A temperature of `raw` 1/256 degrees.
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// The temperature in 1/256 degrees.
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// A temperature of whole `degrees`.
    pub const fn from_degrees(degrees: u8) -> Self {
        Self((degrees as u16) << 8)
    }

    /// The two message bytes, fraction first.
    pub const fn from_bytes(bytes: [u8; 2]) -> Self {
        Self(u16::from_le_bytes(bytes))
    }

    /// The two message bytes, fraction first.
    pub const fn to_bytes(self) -> [u8; 2] {
        self.0.to_le_bytes()
    }

    /// The whole degrees, rounded down.
    pub const fn whole_degrees(self) -> u8 {
        (self.0 >> 8) as u8
    }

    /// The fraction in 1/256 degrees.
    pub const fn fraction(self) -> u8 {
        self.0 as u8
    }

    pub fn degrees(self) -> f32 {
        f32::from(self.0) / 256.0
    }
}

/// Two decimals, rounded down: `45.50`.
impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hundredths = u32::from(self.fraction()) * 100 / 256;
        write!(f, "{}.{hundredths:02}", self.whole_degrees())
    }
}

/// Reply of [`SmcCommand::QueryTemperatures`]. Only `edram` has a known
/// source; which device `temperature0`, `temperature1` and `temperature3`
/// measure is not established.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Temperatures {
    pub temperature0: Temperature,
    pub temperature1: Temperature,
    pub edram: Temperature,
    pub temperature3: Temperature,
    /// The fan speed the SMC requests, in percent; not the measured duty.
    pub fan_speed_percent: u8,
}

impl Temperatures {
    pub const fn from_reply(reply: &SmcMessage) -> Result<Self, SmcDecodeError> {
        if let Err(error) = expect_reply(reply, SmcCommand::QueryTemperatures) {
            return Err(error);
        }

        let bytes = &reply.0;
        Ok(Self {
            temperature0: Temperature::from_bytes([bytes[1], bytes[2]]),
            temperature1: Temperature::from_bytes([bytes[3], bytes[4]]),
            edram: Temperature::from_bytes([bytes[5], bytes[6]]),
            temperature3: Temperature::from_bytes([bytes[7], bytes[8]]),
            fan_speed_percent: bytes[9],
        })
    }

    /// The reply the SMC gives, for a model of the SMC.
    pub const fn to_reply(&self) -> SmcMessage {
        let mut reply = SmcMessage::new(SmcCommand::QueryTemperatures);
        let temperatures = [
            self.temperature0,
            self.temperature1,
            self.edram,
            self.temperature3,
        ];
        let mut index = 0;
        while index < temperatures.len() {
            let bytes = temperatures[index].to_bytes();
            reply.0[1 + index * 2] = bytes[0];
            reply.0[2 + index * 2] = bytes[1];
            index += 1;
        }
        reply.0[9] = self.fan_speed_percent;
        reply
    }
}

/// SMC firmware version, bytes 1-3 of the reply of
/// [`SmcCommand::QueryVersion`]. The three bytes are the build as hex
/// digits: `[0x51, 0x03, 0x01]` is build 510301, which
/// [`Display`](fmt::Display) prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SmcVersion(pub [u8; 3]);

/// Complete reply of [`SmcCommand::QueryVersion`] (`SMC_VERSION_REPLY`),
/// layout of SMC 510301.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VersionReply {
    pub version: SmcVersion,
    /// `BootCause0` and `BootCause1`: the bytes the host last set with SMC
    /// command 0x9c. The kernel reads the boot cause from bits 7-4 of
    /// `boot_cause[1]`.
    pub boot_cause: [u8; 2],
    /// Count of power button presses.
    pub power_button_presses: u8,
}

impl VersionReply {
    pub const fn from_reply(reply: &SmcMessage) -> Result<Self, SmcDecodeError> {
        if let Err(error) = expect_reply(reply, SmcCommand::QueryVersion) {
            return Err(error);
        }

        Ok(Self {
            version: SmcVersion([reply.0[1], reply.0[2], reply.0[3]]),
            boot_cause: [reply.0[4], reply.0[5]],
            power_button_presses: reply.0[6],
        })
    }

    /// The reply the SMC gives, for a model of the SMC.
    pub const fn to_reply(&self) -> SmcMessage {
        let mut reply = SmcMessage::new(SmcCommand::QueryVersion);
        reply.0[1] = self.version.0[0];
        reply.0[2] = self.version.0[1];
        reply.0[3] = self.version.0[2];
        reply.0[4] = self.boot_cause[0];
        reply.0[5] = self.boot_cause[1];
        reply.0[6] = self.power_button_presses;
        reply
    }
}

impl fmt::Display for SmcVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}{:02x}{:02x}", self.0[0], self.0[1], self.0[2])
    }
}

/// Code of the power-up cause in [`PowerUpCause`]. Its values are not
/// established.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PowerUpCauseCode(pub u8);

/// Reply of [`SmcCommand::QueryPowerUpCause`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PowerUpCause {
    pub cause: PowerUpCauseCode,
    pub boot_attempts: u8,
    /// The SMC configuration block is invalid.
    pub config_invalid: bool,
}

impl PowerUpCause {
    const CONFIG_INVALID: u8 = 0x1;

    pub const fn from_reply(reply: &SmcMessage) -> Result<Self, SmcDecodeError> {
        if let Err(error) = expect_reply(reply, SmcCommand::QueryPowerUpCause) {
            return Err(error);
        }

        Ok(Self {
            cause: PowerUpCauseCode(reply.0[1]),
            boot_attempts: reply.0[3],
            config_invalid: reply.0[4] & Self::CONFIG_INVALID != 0,
        })
    }

    /// The reply the SMC gives, for a model of the SMC.
    pub const fn to_reply(&self) -> SmcMessage {
        let mut reply = SmcMessage::new(SmcCommand::QueryPowerUpCause);
        reply.0[1] = self.cause.0;
        reply.0[3] = self.boot_attempts;
        if self.config_invalid {
            reply.0[4] = Self::CONFIG_INVALID;
        }
        reply
    }
}

/// A percentage from 0 to 100.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Percent(u8);

impl Percent {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(100);

    /// `None` above 100.
    pub const fn new(value: u8) -> Option<Self> {
        if value <= Self::MAX.0 {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Percent {
    type Error = crate::UnknownValue<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(crate::UnknownValue { value })
    }
}

impl From<Percent> for u8 {
    fn from(percent: Percent) -> u8 {
        percent.0
    }
}

/// Speed of a fan: [`SmcRequest::SetFanSpeed`] and
/// [`SmcRequest::SetFan2Speed`]. The setting lasts until the console
/// restarts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FanSpeed {
    /// The SMC controls the fan from its temperatures.
    #[default]
    Automatic,
    /// A fixed duty.
    Fixed(Percent),
}

impl FanSpeed {
    const FIXED: u8 = 0x80;
    const PERCENT_MASK: u8 = 0x7f;

    /// Bit 7 set selects a fixed duty of bits 6-0 percent; bit 7 clear,
    /// automatic control.
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Automatic => 0,
            Self::Fixed(percent) => Self::FIXED | percent.get(),
        }
    }

    /// The speed of a request byte. Bits 6-0 of an automatic speed are
    /// ignored, as the SMC ignores them.
    pub const fn from_byte(byte: u8) -> Result<Self, SmcDecodeError> {
        if byte & Self::FIXED == 0 {
            return Ok(Self::Automatic);
        }

        match Percent::new(byte & Self::PERCENT_MASK) {
            Some(percent) => Ok(Self::Fixed(percent)),
            None => Err(SmcDecodeError::InvalidFanSpeed(byte)),
        }
    }
}

flags! {
    /// Fan algorithm of [`SmcRequest::SetFanAlgorithm`].
    pub struct FanAlgorithm(u8) {
        /// The quiet fan curve.
        QUIET = 0x1,
        /// Full fan speed. Dangerous: the SMC keeps it until power-down
        /// and stops polling the temperature sensors meanwhile, so it
        /// reports a sensor timeout and shuts the console down.
        FULL_SPEED = 0x2,
    }
}

/// A pattern of the SMC's power LED table, 2 to 9. What each pattern
/// looks like is not established.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PowerLedPattern(u8);

impl PowerLedPattern {
    pub const FIRST: Self = Self(2);
    pub const LAST: Self = Self(9);

    /// `None` outside 2 to 9.
    pub const fn new(mode: u8) -> Option<Self> {
        if mode >= Self::FIRST.0 && mode <= Self::LAST.0 {
            Some(Self(mode))
        } else {
            None
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Power LED mode of [`SmcRequest::SetPowerLed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerLed {
    /// The SMC controls the LED. The effect of `one_shot` is not
    /// established.
    Automatic {
        one_shot: bool,
    },
    Pattern(PowerLedPattern),
}

impl PowerLed {
    const AUTOMATIC: u8 = 1;
    const ONE_SHOT: u8 = 0x1;

    /// Mode and option bytes.
    pub const fn to_bytes(self) -> [u8; 2] {
        match self {
            Self::Automatic { one_shot } => {
                [Self::AUTOMATIC, if one_shot { Self::ONE_SHOT } else { 0 }]
            }
            Self::Pattern(pattern) => [pattern.get(), 0],
        }
    }

    pub const fn from_bytes(bytes: [u8; 2]) -> Result<Self, SmcDecodeError> {
        if bytes[0] == Self::AUTOMATIC {
            return Ok(Self::Automatic {
                one_shot: bytes[1] & Self::ONE_SHOT != 0,
            });
        }

        match PowerLedPattern::new(bytes[0]) {
            Some(pattern) => Ok(Self::Pattern(pattern)),
            None => Err(SmcDecodeError::InvalidPowerLedMode(bytes[0])),
        }
    }
}

flags! {
    /// The four quadrants of the ring of light. Which bit lights which
    /// physical quadrant is not established.
    pub struct QuadrantMask(u8) {
        QUADRANT_0 = 0x1,
        QUADRANT_1 = 0x2,
        QUADRANT_2 = 0x4,
        QUADRANT_3 = 0x8,
    }
}

/// State of the ring of light of [`SmcRequest::SetRingOfLight`]. There is
/// no colour or brightness control. An override lasts until the console
/// restarts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum RingOfLight {
    /// The system controls the ring.
    #[default]
    System,
    /// The quadrants of the mask are on, the others off.
    Override(QuadrantMask),
}

impl RingOfLight {
    const OVERRIDE: u8 = 0x1;
    const QUADRANT_SHIFT: u8 = 4;

    /// Flags and quadrant bytes. The low nibble of the quadrant byte is
    /// zero, as SMC build 21010d also reads it.
    pub const fn to_bytes(self) -> [u8; 2] {
        match self {
            Self::System => [0, 0],
            Self::Override(mask) => [
                Self::OVERRIDE,
                (mask.bits() & QuadrantMask::all().bits()) << Self::QUADRANT_SHIFT,
            ],
        }
    }

    /// The state of the request bytes; the low nibble of the quadrant
    /// byte is ignored.
    pub const fn from_bytes(bytes: [u8; 2]) -> Self {
        if bytes[0] & Self::OVERRIDE == 0 {
            Self::System
        } else {
            Self::Override(QuadrantMask::from_bits_retain(
                bytes[1] >> Self::QUADRANT_SHIFT,
            ))
        }
    }
}

/// An SMC request with a typed encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SmcRequest {
    /// Reply: [`PowerUpCause`].
    QueryPowerUpCause,
    /// Reply: [`Temperatures`].
    QueryTemperatures,
    /// Reply: [`VersionReply`].
    QueryVersion,
    SetFanAlgorithm(FanAlgorithm),
    SetFanSpeed(FanSpeed),
    /// The second fan output. Only some SMC builds have it (21010d does;
    /// 510301 ignores it).
    SetFan2Speed(FanSpeed),
    SetPowerLed(PowerLed),
    SetRingOfLight(RingOfLight),
}

impl SmcRequest {
    pub const fn command(self) -> SmcCommand {
        match self {
            Self::QueryPowerUpCause => SmcCommand::QueryPowerUpCause,
            Self::QueryTemperatures => SmcCommand::QueryTemperatures,
            Self::QueryVersion => SmcCommand::QueryVersion,
            Self::SetFanAlgorithm(_) => SmcCommand::SetFanAlgorithm,
            Self::SetFanSpeed(_) => SmcCommand::SetFanSpeed,
            Self::SetFan2Speed(_) => SmcCommand::SetFan2Speed,
            Self::SetPowerLed(_) => SmcCommand::SetPowerLed,
            Self::SetRingOfLight(_) => SmcCommand::SetRingOfLight,
        }
    }

    /// Whether the SMC replies: true for the queries.
    pub const fn has_reply(self) -> bool {
        self.command().is_query()
    }

    pub const fn to_message(self) -> SmcMessage {
        let mut message = SmcMessage::new(self.command());
        let data = match self {
            Self::QueryPowerUpCause | Self::QueryTemperatures | Self::QueryVersion => [0, 0],
            Self::SetFanAlgorithm(algorithm) => [algorithm.bits(), 0],
            Self::SetFanSpeed(speed) | Self::SetFan2Speed(speed) => [speed.to_byte(), 0],
            Self::SetPowerLed(led) => led.to_bytes(),
            Self::SetRingOfLight(ring) => ring.to_bytes(),
        };
        message.0[1] = data[0];
        message.0[2] = data[1];
        message
    }

    /// The request a message carries, for a model of the SMC.
    pub const fn from_message(message: &SmcMessage) -> Result<Self, SmcDecodeError> {
        let data = [message.0[1], message.0[2]];
        let Some(command) = message.command() else {
            return Err(SmcDecodeError::UnsupportedCommand(message.0[0]));
        };

        Ok(match command {
            SmcCommand::QueryPowerUpCause => Self::QueryPowerUpCause,
            SmcCommand::QueryTemperatures => Self::QueryTemperatures,
            SmcCommand::QueryVersion => Self::QueryVersion,
            SmcCommand::SetFanAlgorithm => {
                Self::SetFanAlgorithm(FanAlgorithm::from_bits_retain(data[0]))
            }
            SmcCommand::SetFanSpeed => match FanSpeed::from_byte(data[0]) {
                Ok(speed) => Self::SetFanSpeed(speed),
                Err(error) => return Err(error),
            },
            SmcCommand::SetFan2Speed => match FanSpeed::from_byte(data[0]) {
                Ok(speed) => Self::SetFan2Speed(speed),
                Err(error) => return Err(error),
            },
            SmcCommand::SetPowerLed => match PowerLed::from_bytes(data) {
                Ok(led) => Self::SetPowerLed(led),
                Err(error) => return Err(error),
            },
            SmcCommand::SetRingOfLight => Self::SetRingOfLight(RingOfLight::from_bytes(data)),
            _ => return Err(SmcDecodeError::UnsupportedCommand(message.0[0])),
        })
    }
}

impl From<SmcRequest> for SmcMessage {
    fn from(request: SmcRequest) -> Self {
        request.to_message()
    }
}

impl TryFrom<&SmcMessage> for SmcRequest {
    type Error = SmcDecodeError;

    fn try_from(message: &SmcMessage) -> Result<Self, SmcDecodeError> {
        Self::from_message(message)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::ToString;

    use super::*;

    fn message(bytes: &[u8]) -> SmcMessage {
        let mut message = SmcMessage::default();
        message.0[..bytes.len()].copy_from_slice(bytes);
        message
    }

    #[test]
    fn temperatures_decode_fraction_byte_first() {
        let reply = message(&[0x07, 0x80, 0x2d, 0x40, 0x30, 0x00, 0x41, 0xc0, 0x28, 0x37]);
        let temperatures = Temperatures::from_reply(&reply).unwrap();

        assert_eq!(temperatures.temperature0, Temperature::from_raw(0x2d80));
        assert_eq!(temperatures.temperature1.whole_degrees(), 0x30);
        assert_eq!(temperatures.temperature1.fraction(), 0x40);
        assert_eq!(temperatures.edram, Temperature::from_degrees(0x41));
        assert_eq!(temperatures.temperature3.raw(), 0x28c0);
        assert_eq!(temperatures.fan_speed_percent, 0x37);
        assert_eq!(temperatures.temperature0.to_string(), "45.50");
        assert_eq!(temperatures.temperature3.degrees(), 40.75);
        assert_eq!(temperatures.to_reply(), reply);
    }

    #[test]
    fn replies_check_their_command() {
        let reply = message(&[0x12, 0x51, 0x03, 0x01, 0x00, 0x20, 0x03]);

        assert_eq!(
            Temperatures::from_reply(&reply),
            Err(SmcDecodeError::WrongCommand {
                expected: SmcCommand::QueryTemperatures,
                found: 0x12,
            })
        );
        let decoded = VersionReply::from_reply(&reply).unwrap();
        assert_eq!(
            decoded,
            VersionReply {
                version: SmcVersion([0x51, 0x03, 0x01]),
                boot_cause: [0x00, 0x20],
                power_button_presses: 3,
            }
        );
        assert_eq!(decoded.version.to_string(), "510301");
        assert_eq!(decoded.to_reply(), reply);
    }

    #[test]
    fn power_up_cause_round_trips() {
        let reply = message(&[0x01, 0x11, 0x00, 0x02, 0x01]);
        let cause = PowerUpCause::from_reply(&reply).unwrap();

        assert_eq!(
            cause,
            PowerUpCause {
                cause: PowerUpCauseCode(0x11),
                boot_attempts: 2,
                config_invalid: true,
            }
        );
        assert_eq!(cause.to_reply(), reply);
    }

    #[test]
    fn percent_is_at_most_100() {
        assert_eq!(Percent::new(100), Some(Percent::MAX));
        assert_eq!(Percent::new(101), None);
        assert_eq!(
            Percent::try_from(200),
            Err(crate::UnknownValue { value: 200 })
        );
    }

    #[test]
    fn fan_speed_bytes() {
        let half = FanSpeed::Fixed(Percent::new(50).unwrap());

        assert_eq!(FanSpeed::Automatic.to_byte(), 0x00);
        assert_eq!(half.to_byte(), 0xb2);
        assert_eq!(FanSpeed::Fixed(Percent::MAX).to_byte(), 0xe4);
        assert_eq!(FanSpeed::from_byte(0xb2), Ok(half));
        assert_eq!(FanSpeed::from_byte(0x32), Ok(FanSpeed::Automatic));
        assert_eq!(
            FanSpeed::from_byte(0xe5),
            Err(SmcDecodeError::InvalidFanSpeed(0xe5))
        );
    }

    #[test]
    fn ring_of_light_bytes() {
        let ring = RingOfLight::Override(QuadrantMask::QUADRANT_0 | QuadrantMask::QUADRANT_3);

        assert_eq!(RingOfLight::System.to_bytes(), [0, 0]);
        assert_eq!(ring.to_bytes(), [0x01, 0x90]);
        assert_eq!(
            RingOfLight::Override(QuadrantMask::from_bits_retain(0xff)).to_bytes(),
            [0x01, 0xf0]
        );
        assert_eq!(RingOfLight::from_bytes([0x01, 0x95]), ring);
        assert_eq!(RingOfLight::from_bytes([0x00, 0xf0]), RingOfLight::System);
    }

    #[test]
    fn power_led_bytes() {
        let pattern = PowerLed::Pattern(PowerLedPattern::new(5).unwrap());

        assert_eq!(PowerLed::Automatic { one_shot: true }.to_bytes(), [1, 1]);
        assert_eq!(pattern.to_bytes(), [5, 0]);
        assert_eq!(PowerLed::from_bytes([5, 0]), Ok(pattern));
        assert_eq!(PowerLedPattern::new(1), None);
        assert_eq!(PowerLedPattern::new(10), None);
        assert_eq!(
            PowerLed::from_bytes([0, 0]),
            Err(SmcDecodeError::InvalidPowerLedMode(0))
        );
    }

    #[test]
    fn requests_round_trip_through_messages() {
        let requests = [
            SmcRequest::QueryPowerUpCause,
            SmcRequest::QueryTemperatures,
            SmcRequest::QueryVersion,
            SmcRequest::SetFanAlgorithm(FanAlgorithm::QUIET),
            SmcRequest::SetFanSpeed(FanSpeed::Fixed(Percent::new(40).unwrap())),
            SmcRequest::SetFan2Speed(FanSpeed::Automatic),
            SmcRequest::SetPowerLed(PowerLed::Automatic { one_shot: false }),
            SmcRequest::SetRingOfLight(RingOfLight::Override(QuadrantMask::all())),
            SmcRequest::SetRingOfLight(RingOfLight::System),
        ];
        for request in requests {
            let message = request.to_message();
            assert_eq!(message.0[0], request.command().raw());
            assert_eq!(message.0[3..], [0; 13]);
            assert_eq!(SmcRequest::from_message(&message), Ok(request));
        }

        assert_eq!(
            SmcRequest::SetFanSpeed(FanSpeed::Fixed(Percent::new(40).unwrap())).to_message(),
            message(&[0x89, 0xa8])
        );
        assert_eq!(
            SmcRequest::SetRingOfLight(RingOfLight::Override(QuadrantMask::QUADRANT_1))
                .to_message(),
            message(&[0x99, 0x01, 0x20])
        );
        assert!(SmcRequest::QueryVersion.has_reply());
        assert!(!SmcRequest::SetPowerLed(PowerLed::Automatic { one_shot: false }).has_reply());
        assert_eq!(
            SmcRequest::from_message(&message(&[0x11])),
            Err(SmcDecodeError::UnsupportedCommand(0x11))
        );
        assert_eq!(
            SmcRequest::from_message(&message(&[0x42])),
            Err(SmcDecodeError::UnsupportedCommand(0x42))
        );
    }
}
