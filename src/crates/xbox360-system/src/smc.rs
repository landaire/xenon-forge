//! The system management controller (SMC): temperatures, firmware version,
//! fan speeds, the power LED and the ring of light, through
//! `HalSendSMCMessage`.
//!
//! Fan, LED and ring settings live in SMC memory and last until the
//! console restarts. The request encodings follow SMC firmware 510301;
//! [`version`] names the console's build.

use xbox360_sys::xboxkrnl;
pub use xbox360_sys::xboxkrnl::FanSpeed;
pub use xbox360_sys::xboxkrnl::Percent;
pub use xbox360_sys::xboxkrnl::PowerLed;
pub use xbox360_sys::xboxkrnl::PowerLedPattern;
pub use xbox360_sys::xboxkrnl::PowerUpCause;
pub use xbox360_sys::xboxkrnl::QuadrantMask;
pub use xbox360_sys::xboxkrnl::RingOfLight;
pub use xbox360_sys::xboxkrnl::SmcDecodeError;
pub use xbox360_sys::xboxkrnl::SmcMessage;
pub use xbox360_sys::xboxkrnl::SmcRequest;
pub use xbox360_sys::xboxkrnl::SmcVersion;
pub use xbox360_sys::xboxkrnl::Temperature;
pub use xbox360_sys::xboxkrnl::Temperatures;
pub use xbox360_sys::xboxkrnl::VersionReply;

/// Send `request` and return the reply of a query, `None` for a command
/// without a reply. Only the queries of [`SmcRequest`], which the SMC
/// answers, wait for a reply.
pub fn send(request: SmcRequest) -> Option<SmcMessage> {
    let message = request.to_message();
    if !request.has_reply() {
        // SAFETY: the message is a live local, and a command without a
        // reply takes a null reply.
        unsafe { xboxkrnl::HalSendSMCMessage(&message, core::ptr::null_mut()) };
        return None;
    }

    let mut reply = SmcMessage::default();

    // SAFETY: both messages are live 16-byte locals, and the SMC answers
    // every query of SmcRequest, so the call returns.
    unsafe { xboxkrnl::HalSendSMCMessage(&message, &mut reply) };
    Some(reply)
}

fn query(request: SmcRequest) -> SmcMessage {
    match send(request) {
        Some(reply) => reply,
        None => unreachable!("{request:?} is a query"),
    }
}

/// The temperatures and the requested fan speed.
pub fn temperatures() -> Result<Temperatures, SmcDecodeError> {
    Temperatures::from_reply(&query(SmcRequest::QueryTemperatures))
}

/// The SMC firmware version, with the boot cause bytes and the power
/// button press count of the same reply.
pub fn version() -> Result<VersionReply, SmcDecodeError> {
    VersionReply::from_reply(&query(SmcRequest::QueryVersion))
}

/// The cause of the last power-up.
pub fn power_up_cause() -> Result<PowerUpCause, SmcDecodeError> {
    PowerUpCause::from_reply(&query(SmcRequest::QueryPowerUpCause))
}

/// Set the speed of the fan.
pub fn set_fan_speed(speed: FanSpeed) {
    send(SmcRequest::SetFanSpeed(speed));
}

/// Set the speed of the second fan output. Only some SMC builds have it
/// (21010d does); others ignore the request.
pub fn set_secondary_fan_speed(speed: FanSpeed) {
    send(SmcRequest::SetFan2Speed(speed));
}

/// Set the ring of light; [`RingOfLight::System`] returns it to the system.
pub fn set_ring_of_light(ring: RingOfLight) {
    send(SmcRequest::SetRingOfLight(ring));
}

/// Set the power LED mode.
pub fn set_power_led(led: PowerLed) {
    send(SmcRequest::SetPowerLed(led));
}
