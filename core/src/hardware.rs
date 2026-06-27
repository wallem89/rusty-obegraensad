use crate::display::ObegraensadDisplay;

/// Board-specific output for the OBEGRÄNSAD serial LED-driver chain.
///
/// Core animation code only produces an `ObegraensadDisplay` brightness frame.
/// Board crates own the concrete transport, latch timing, enable pin polarity,
/// and whether brightness is displayed via thresholding or PWM.
pub trait DisplayDriver {
    type Error;

    fn write_frame(
        &mut self,
        display: &ObegraensadDisplay,
        pwm_phase: u8,
    ) -> Result<(), Self::Error>;

    fn latch(&mut self) -> Result<(), Self::Error>;

    fn set_enabled(&mut self, enabled: bool) -> Result<(), Self::Error>;
}

/// Board-specific user input used to select the active animation.
pub trait AnimationSelect {
    type Error;

    fn is_selected(&mut self) -> Result<bool, Self::Error>;
}
