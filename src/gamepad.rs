use std::thread;
use std::time::Duration;
use vigem_client::{Client, TargetId, Xbox360Wired, XGamepad};

pub struct GamepadManager {
    target: Xbox360Wired<Client>,
}

impl GamepadManager {
    pub fn new() -> Result<Self, String> {
        let client = Client::connect()
            .map_err(|e| format!("ViGEmBus connection failed: {:?}. Is ViGEmBus driver installed?", e))?;
        
        let mut target = Xbox360Wired::new(client, TargetId::XBOX360_WIRED);
        target
            .plugin()
            .map_err(|e| format!("Failed to plug in virtual controller: {:?}", e))?;
        
        target
            .wait_ready()
            .map_err(|e| format!("Controller handshake timeout: {:?}", e))?;

        Ok(Self { target })
    }

    pub fn pulse(&mut self, direction_step: usize) -> Result<(), String> {
        // Alternating right thumbstick deflections (controls camera pan in Roblox, character never moves)
        let directions: [(i16, i16); 4] = [
            (22_000, 22_000),   // UP-RIGHT
            (-22_000, 22_000),  // UP-LEFT
            (22_000, -22_000),  // DOWN-RIGHT
            (-22_000, -22_000), // DOWN-LEFT
        ];

        let (rx, ry) = directions[direction_step % directions.len()];

        let mut report = XGamepad::default();
        report.thumb_rx = rx;
        report.thumb_ry = ry;
        self.target
            .update(&report)
            .map_err(|e| format!("Failed to send gamepad report: {:?}", e))?;

        thread::sleep(Duration::from_millis(200));

        // Re-center to absolute neutral (0, 0)
        report.thumb_rx = 0;
        report.thumb_ry = 0;
        self.target
            .update(&report)
            .map_err(|e| format!("Failed to reset gamepad neutral: {:?}", e))?;

        Ok(())
    }
}
