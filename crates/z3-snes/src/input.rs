//! Exact controller shift-register behavior from `snes/input.c`.

/// Standard controller type selected by `input_init` in the C port.
pub const INPUT_CONTROLLER: u8 = 1;

/// One SNES controller port's latch and serial shift state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Input {
    input_type: u8,
    latch_line: bool,
    current_state: u16,
    latched_state: u16,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    /// Creates a standard controller with no buttons pressed.
    ///
    /// C leaves the latch fields uninitialized until reset. Rust initializes
    /// them to the state produced by `input_reset`, avoiding undefined state
    /// while preserving all behavior after normal SNES initialization.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            input_type: INPUT_CONTROLLER,
            latch_line: false,
            current_state: 0,
            latched_state: 0,
        }
    }

    /// Resets the latch line and serial register without changing live input.
    pub const fn reset(&mut self) {
        self.latch_line = false;
        self.latched_state = 0;
    }

    /// Performs one input cycle. While latch is high, the live state is copied
    /// on every cycle, exactly as in the C implementation.
    pub const fn cycle(&mut self) {
        if self.latch_line {
            self.latched_state = self.current_state;
        }
    }

    /// Reads the low serial bit, shifts right, and fills the high bit with one.
    pub const fn read(&mut self) -> u8 {
        let result = (self.latched_state & 1) as u8;
        self.latched_state >>= 1;
        self.latched_state |= 0x8000;
        result
    }

    /// Returns the raw controller type field.
    #[must_use]
    pub const fn input_type(&self) -> u8 {
        self.input_type
    }

    /// Sets the controller type field for state restoration.
    pub const fn set_input_type(&mut self, input_type: u8) {
        self.input_type = input_type;
    }

    /// Returns the current state of the latch line.
    #[must_use]
    pub const fn latch_line(&self) -> bool {
        self.latch_line
    }

    /// Drives the latch line. The current button state is captured by
    /// [`Self::cycle`], not by changing the line itself.
    pub const fn set_latch_line(&mut self, high: bool) {
        self.latch_line = high;
    }

    /// Returns the live 16-bit button state.
    #[must_use]
    pub const fn current_state(&self) -> u16 {
        self.current_state
    }

    /// Updates the live 16-bit button state.
    pub const fn set_current_state(&mut self, state: u16) {
        self.current_state = state;
    }

    /// Returns the internal serial shift register for save states and parity
    /// diagnostics.
    #[must_use]
    pub const fn latched_state(&self) -> u16 {
        self.latched_state
    }

    /// Restores the internal serial shift register from a save state.
    pub const fn set_latched_state(&mut self, state: u16) {
        self.latched_state = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialization_and_reset_match_c_lifecycle() {
        let mut input = Input::new();
        assert_eq!(input.input_type(), INPUT_CONTROLLER);
        assert!(!input.latch_line());
        assert_eq!(input.current_state(), 0);
        assert_eq!(input.latched_state(), 0);

        input.set_current_state(0xabcd);
        input.set_latched_state(0x1234);
        input.set_latch_line(true);
        input.set_input_type(9);
        input.reset();

        assert_eq!(input.input_type(), 9);
        assert_eq!(input.current_state(), 0xabcd);
        assert_eq!(input.latched_state(), 0);
        assert!(!input.latch_line());
    }

    #[test]
    fn cycle_only_captures_while_latch_is_high() {
        let mut input = Input::new();
        input.set_current_state(0xa55a);
        input.cycle();
        assert_eq!(input.latched_state(), 0);

        input.set_latch_line(true);
        input.cycle();
        assert_eq!(input.latched_state(), 0xa55a);

        input.set_current_state(0x0f0f);
        input.cycle();
        assert_eq!(input.latched_state(), 0x0f0f);

        input.set_latch_line(false);
        input.set_current_state(0xffff);
        input.cycle();
        assert_eq!(input.latched_state(), 0x0f0f);
    }

    #[test]
    fn read_shifts_low_bit_first_and_fills_with_ones() {
        let mut input = Input::new();
        input.set_latched_state(0xa55a);

        let bits: Vec<u8> = (0..16).map(|_| input.read()).collect();
        let expected: Vec<u8> = (0..16).map(|bit| ((0xa55au16 >> bit) & 1) as u8).collect();
        assert_eq!(bits, expected);
        assert_eq!(input.latched_state(), 0xffff);

        assert_eq!(input.read(), 1);
        assert_eq!(input.latched_state(), 0xffff);
    }

    #[test]
    fn reads_do_not_recapture_until_cycle_runs() {
        let mut input = Input::new();
        input.set_current_state(0x0001);
        input.set_latch_line(true);

        input.cycle();
        assert_eq!(input.read(), 1);
        assert_eq!(input.read(), 0);

        // Latch being high alone does not change the shift register.
        assert_eq!(input.read(), 0);
        input.cycle();
        assert_eq!(input.read(), 1);
    }
}
