use crate::data::GamepadState;
use log::{warn, debug, trace};

pub trait InputExecutor: Send {
    fn submit(&mut self, state: &GamepadState);
    fn shutdown(&mut self);
    fn set_rumble_callback(&mut self, callback: Box<dyn Fn(u8, u8) + Send>);
}

#[cfg(target_os = "windows")]
pub struct XInputExecutor {
    client: vigem_client::Client,
    target: vigem_client::Xbox360Wired<vigem_client::Client>,
}

#[cfg(target_os = "windows")]
impl XInputExecutor {
    pub fn new() -> Result<Self, vigem_client::Error> {
        let client = vigem_client::Client::connect()?;
        let mut target = vigem_client::Xbox360Wired::new(
            client.try_clone().unwrap(), 
            vigem_client::TargetId::XBOX360_WIRED
        );
        target.plugin()?;
        
        Ok(Self { client, target })
    }

    fn deadzone(v: i16) -> i16 {
        if v.abs() < 4000 { 0 } else { v }
    }
}

#[cfg(target_os = "windows")]
impl InputExecutor for XInputExecutor {
    fn submit(&mut self, state: &GamepadState) {
        let report = vigem_client::XGamepad {
            buttons: vigem_client::XButtons(state.buttons),
            left_trigger: state.lt,
            right_trigger: state.rt,
            thumb_lx: Self::deadzone(state.lx),
            thumb_ly: Self::deadzone(state.ly),
            thumb_rx: Self::deadzone(state.rx),
            thumb_ry: Self::deadzone(state.ry),
        };
        let _ = self.target.update(&report);
    }

    fn shutdown(&mut self) {
        let _ = self.target.unplug();
    }

    fn set_rumble_callback(&mut self, callback: Box<dyn Fn(u8, u8) + Send>) {
        match self.target.request_notification() {
            Ok(notification) => {
                debug!("Rumble notification thread registered");
                notification.spawn_thread(move |_notif, data| {
                    trace!("Rumble: large={} small={}", data.large_motor, data.small_motor);
                    callback(data.large_motor, data.small_motor);
                });
            }
            Err(e) => {
                warn!("Failed to request rumble notification: {:?}", e);
            }
        }
    }
}