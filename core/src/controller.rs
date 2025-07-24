use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ControllerButton: u8 {
        const A      = 0b00000001;
        const B      = 0b00000010;
        const SELECT = 0b00000100;
        const START  = 0b00001000;
        const UP     = 0b00010000;
        const DOWN   = 0b00100000;
        const LEFT   = 0b01000000;
        const RIGHT  = 0b10000000;
    }
}

#[derive(Debug, Clone)]
pub struct Controller {
    buttons: ControllerButton,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            buttons: ControllerButton::empty(),
        }
    }
    
    pub fn set_button(&mut self, button: ControllerButton, pressed: bool) {
        if pressed {
            self.buttons.insert(button);
        } else {
            self.buttons.remove(button);
        }
    }
    
    pub fn set_buttons(&mut self, buttons: ControllerButton) {
        self.buttons = buttons;
    }
    
    pub fn get_state(&self) -> u8 {
        self.buttons.bits()
    }
    
    pub fn is_pressed(&self, button: ControllerButton) -> bool {
        self.buttons.contains(button)
    }
    
    pub fn clear(&mut self) {
        self.buttons = ControllerButton::empty();
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_controller_buttons() {
        let mut controller = Controller::new();
        
        // Test individual button presses
        controller.set_button(ControllerButton::A, true);
        assert!(controller.is_pressed(ControllerButton::A));
        assert!(!controller.is_pressed(ControllerButton::B));
        assert_eq!(controller.get_state(), 0x01);
        
        // Test multiple buttons
        controller.set_button(ControllerButton::B, true);
        controller.set_button(ControllerButton::START, true);
        assert!(controller.is_pressed(ControllerButton::A));
        assert!(controller.is_pressed(ControllerButton::B));
        assert!(controller.is_pressed(ControllerButton::START));
        assert_eq!(controller.get_state(), 0x0B);
        
        // Test button release
        controller.set_button(ControllerButton::A, false);
        assert!(!controller.is_pressed(ControllerButton::A));
        assert!(controller.is_pressed(ControllerButton::B));
        assert_eq!(controller.get_state(), 0x0A);
        
        // Test clear
        controller.clear();
        assert_eq!(controller.get_state(), 0x00);
    }
    
    #[test]
    fn test_controller_set_buttons() {
        let mut controller = Controller::new();
        
        // Set multiple buttons at once
        controller.set_buttons(ControllerButton::UP | ControllerButton::A);
        assert!(controller.is_pressed(ControllerButton::UP));
        assert!(controller.is_pressed(ControllerButton::A));
        assert!(!controller.is_pressed(ControllerButton::DOWN));
        assert_eq!(controller.get_state(), 0x11);
    }
}