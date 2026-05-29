#![no_std]

/// The four states a oneshot key can be in.
///
/// This is a direct port of Callum's oneshot implementation:
/// no timers, purely state-machine driven.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OneshotState {
    /// Key is up and not queued — inactive.
    UpUnqueued,
    /// Key is up but queued — will apply to the next non-ignored keypress.
    UpQueued,
    /// Key is held down but hasn't been "used" by another key yet.
    DownUnused,
    /// Key is held down and has been used (another key was pressed while held).
    DownUsed,
}

impl Default for OneshotState {
    fn default() -> Self {
        Self::UpUnqueued
    }
}

/// Represents a key event (press or release) with a keycode.
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub keycode: u16,
    pub pressed: bool,
}

/// A single oneshot modifier instance.
///
/// Each modifier you want (Shift, Ctrl, Alt, Gui) gets its own `OneshotMod`.
pub struct OneshotMod {
    pub state: OneshotState,
    /// The modifier keycode this oneshot controls (e.g. left shift).
    pub modifier: u8,
    /// The trigger keycode that activates this oneshot.
    pub trigger: u16,
}

/// Result of processing a oneshot event — tells the caller what action to take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OneshotAction {
    /// No action needed.
    None,
    /// Register (press) the modifier.
    Register(u8),
    /// Unregister (release) the modifier.
    Unregister(u8),
}

impl OneshotMod {
    pub const fn new(modifier: u8, trigger: u16) -> Self {
        Self {
            state: OneshotState::UpUnqueued,
            modifier,
            trigger,
        }
    }

    /// Process a key event and return what action the firmware should take.
    ///
    /// The caller must provide two predicates:
    /// - `is_cancel`: returns true for keycodes that should cancel a queued oneshot
    /// - `is_ignored`: returns true for keycodes that should not count as "using" the mod
    ///   (typically other mods and layer keys, allowing oneshot stacking)
    pub fn update(
        &mut self,
        event: KeyEvent,
        is_cancel: impl Fn(u16) -> bool,
        is_ignored: impl Fn(u16) -> bool,
    ) -> OneshotAction {
        if event.keycode == self.trigger {
            if event.pressed {
                // Trigger keydown
                let action = if self.state == OneshotState::UpUnqueued {
                    OneshotAction::Register(self.modifier)
                } else {
                    OneshotAction::None
                };
                self.state = OneshotState::DownUnused;
                action
            } else {
                // Trigger keyup
                match self.state {
                    OneshotState::DownUnused => {
                        // Wasn't used while held — queue it for next keypress
                        self.state = OneshotState::UpQueued;
                        OneshotAction::None
                    }
                    OneshotState::DownUsed => {
                        // Was used while held — unregister now
                        self.state = OneshotState::UpUnqueued;
                        OneshotAction::Unregister(self.modifier)
                    }
                    _ => OneshotAction::None,
                }
            }
        } else if event.pressed {
            // Non-trigger keydown
            if is_cancel(event.keycode) && self.state != OneshotState::UpUnqueued {
                self.state = OneshotState::UpUnqueued;
                OneshotAction::Unregister(self.modifier)
            } else {
                OneshotAction::None
            }
        } else {
            // Non-trigger keyup
            if !is_ignored(event.keycode) {
                match self.state {
                    OneshotState::DownUnused => {
                        self.state = OneshotState::DownUsed;
                        OneshotAction::None
                    }
                    OneshotState::UpQueued => {
                        self.state = OneshotState::UpUnqueued;
                        OneshotAction::Unregister(self.modifier)
                    }
                    _ => OneshotAction::None,
                }
            } else {
                OneshotAction::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOD_LSHIFT: u8 = 0xE1;
    const TRIGGER: u16 = 0xFF00;
    const KEY_A: u16 = 0x04;
    const LAYER_KEY: u16 = 0xFE00;

    fn is_cancel(kc: u16) -> bool {
        kc == LAYER_KEY
    }

    fn is_ignored(kc: u16) -> bool {
        kc == LAYER_KEY || kc == TRIGGER
    }

    #[test]
    fn tap_queues_then_releases_on_next_key() {
        let mut osm = OneshotMod::new(MOD_LSHIFT, TRIGGER);

        // Press trigger
        let a = osm.update(KeyEvent { keycode: TRIGGER, pressed: true }, is_cancel, is_ignored);
        assert_eq!(a, OneshotAction::Register(MOD_LSHIFT));

        // Release trigger — should queue
        let a = osm.update(KeyEvent { keycode: TRIGGER, pressed: false }, is_cancel, is_ignored);
        assert_eq!(a, OneshotAction::None);
        assert_eq!(osm.state, OneshotState::UpQueued);

        // Press A
        let a = osm.update(KeyEvent { keycode: KEY_A, pressed: true }, is_cancel, is_ignored);
        assert_eq!(a, OneshotAction::None);

        // Release A — should unregister
        let a = osm.update(KeyEvent { keycode: KEY_A, pressed: false }, is_cancel, is_ignored);
        assert_eq!(a, OneshotAction::Unregister(MOD_LSHIFT));
    }

    #[test]
    fn hold_and_use_unregisters_on_trigger_release() {
        let mut osm = OneshotMod::new(MOD_LSHIFT, TRIGGER);

        // Press trigger
        osm.update(KeyEvent { keycode: TRIGGER, pressed: true }, is_cancel, is_ignored);
        // Press A while holding trigger
        osm.update(KeyEvent { keycode: KEY_A, pressed: true }, is_cancel, is_ignored);
        // Release A — marks as used
        osm.update(KeyEvent { keycode: KEY_A, pressed: false }, is_cancel, is_ignored);
        assert_eq!(osm.state, OneshotState::DownUsed);

        // Release trigger — should unregister
        let a = osm.update(KeyEvent { keycode: TRIGGER, pressed: false }, is_cancel, is_ignored);
        assert_eq!(a, OneshotAction::Unregister(MOD_LSHIFT));
    }
}
