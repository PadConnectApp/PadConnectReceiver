#[cfg(target_os = "linux")]
use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder, UinputAbsSetup},
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key,
};
#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
#[cfg(target_os = "linux")]
use std::thread;
#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "linux")]
use libc;

#[cfg(target_os = "linux")]
pub struct UinputExecutor {
    device: Arc<Mutex<VirtualDevice>>,
    rumble_thread: Option<thread::JoinHandle<()>>,
}

#[cfg(target_os = "linux")]
impl UinputExecutor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut keys = AttributeSet::<Key>::new();
        // ABXY
        keys.insert(Key::BTN_SOUTH); // A
        keys.insert(Key::BTN_EAST);  // B
        keys.insert(Key::BTN_NORTH); // X
        keys.insert(Key::BTN_WEST);  // Y
        // Other buttons
        keys.insert(Key::BTN_TL);
        keys.insert(Key::BTN_TR);
        keys.insert(Key::BTN_THUMBL);
        keys.insert(Key::BTN_THUMBR);
        keys.insert(Key::BTN_SELECT);
        keys.insert(Key::BTN_START);
        keys.insert(Key::BTN_MODE); // Xbox/Guide button

        let device = VirtualDeviceBuilder::new()?
            .name("PadConnect")
            .with_keys(&keys)?
            .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisType::ABS_X, AbsInfo::new(0, -32768, 32767, 16, 128)))?
            .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, AbsInfo::new(0, -32768, 32767, 16, 128)))?
            .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisType::ABS_RX, AbsInfo::new(0, -32768, 32767, 16, 128)))?
            .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisType::ABS_RY, AbsInfo::new(0, -32768, 32767, 16, 128)))?
            .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisType::ABS_Z, AbsInfo::new(0, 0, 255, 0, 0)))? // LT
            .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisType::ABS_RZ, AbsInfo::new(0, 0, 255, 0, 0)))? // RT
            .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisType::ABS_HAT0X, AbsInfo::new(0, -1, 1, 0, 0)))? // D-Pad X
            .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisType::ABS_HAT0Y, AbsInfo::new(0, -1, 1, 0, 0)))? // D-Pad Y
            .build()?;

        Ok(Self {
            device: Arc::new(Mutex::new(device)),
            rumble_thread: None,
        })
    }

    fn deadzone(v: i16) -> i32 {
        if v.abs() < 4000 { 0 } else { v as i32 }
    }
}

#[cfg(target_os = "linux")]
impl InputExecutor for UinputExecutor {
    fn submit(&mut self, state: &GamepadState) {
        let mut events = Vec::with_capacity(16);
        let b = state.buttons;

        // XInput Bitmask map:
        // 0x0001: Dpad Up, 0x0002: Dpad Down, 0x0004: Dpad Left, 0x0008: Dpad Right
        // 0x0010: Start, 0x0020: Back, 0x0040: LThumb, 0x0080: RThumb
        // 0x0100: LShoulder, 0x0200: RShoulder
        // 0x1000: A, 0x2000: B, 0x4000: X, 0x8000: Y
        
        events.push(InputEvent::new(EventType::KEY, Key::BTN_SOUTH.code(), if b & 0x1000 != 0 { 1 } else { 0 }));
        events.push(InputEvent::new(EventType::KEY, Key::BTN_EAST.code(),  if b & 0x2000 != 0 { 1 } else { 0 }));
        events.push(InputEvent::new(EventType::KEY, Key::BTN_NORTH.code(), if b & 0x4000 != 0 { 1 } else { 0 }));
        events.push(InputEvent::new(EventType::KEY, Key::BTN_WEST.code(),  if b & 0x8000 != 0 { 1 } else { 0 }));
        
        events.push(InputEvent::new(EventType::KEY, Key::BTN_TL.code(),     if b & 0x0100 != 0 { 1 } else { 0 }));
        events.push(InputEvent::new(EventType::KEY, Key::BTN_TR.code(),     if b & 0x0200 != 0 { 1 } else { 0 }));
        events.push(InputEvent::new(EventType::KEY, Key::BTN_THUMBL.code(), if b & 0x0040 != 0 { 1 } else { 0 }));
        events.push(InputEvent::new(EventType::KEY, Key::BTN_THUMBR.code(), if b & 0x0080 != 0 { 1 } else { 0 }));
        events.push(InputEvent::new(EventType::KEY, Key::BTN_START.code(),  if b & 0x0010 != 0 { 1 } else { 0 }));
        events.push(InputEvent::new(EventType::KEY, Key::BTN_SELECT.code(), if b & 0x0020 != 0 { 1 } else { 0 }));

        // D-Pad
        let dpad_x = if b & 0x0008 != 0 { 1 } else if b & 0x0004 != 0 { -1 } else { 0 };
        let dpad_y = if b & 0x0002 != 0 { 1 } else if b & 0x0001 != 0 { -1 } else { 0 };
        events.push(InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_HAT0X.0, dpad_x));
        events.push(InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_HAT0Y.0, dpad_y));

        // Triggers
        events.push(InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_Z.0, state.lt as i32));
        events.push(InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_RZ.0, state.rt as i32));

        // Thumbsticks
        events.push(InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_X.0, Self::deadzone(state.lx)));
        events.push(InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_Y.0, -Self::deadzone(state.ly)));
        events.push(InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_RX.0, Self::deadzone(state.rx)));
        events.push(InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_RY.0, -Self::deadzone(state.ry)));

        if let Ok(mut device) = self.device.lock() {
            let _ = device.emit(&events);
        }
    }

    fn shutdown(&mut self) {
        // uinput device is automatically destroyed when the fd is closed on drop
    }

    fn set_rumble_callback(&mut self, callback: Box<dyn Fn(u8, u8) + Send>) {
        let device_arc = Arc::clone(&self.device);
        
        let handle = thread::spawn(move || {
            debug!("Rumble notification thread registered (Linux uinput)");
            
            let fd = {
                let dev = device_arc.lock().unwrap();
                dev.as_raw_fd()
            };

            let mut ev: libc::input_event = unsafe { std::mem::zeroed() };
            loop {
                // This is a placeholder for reading rumble events from the uinput device.
                let bytes_read = unsafe {
                    libc::read(
                        fd,
                        &mut ev as *mut _ as *mut libc::c_void,
                        std::mem::size_of::<libc::input_event>(),
                    )
                };

                if bytes_read == std::mem::size_of::<libc::input_event>() as isize {
                    if ev.type_ == 0x15 { 
                        trace!("Rumble Event Received");
                        callback(255, 255); 
                    }
                } else if bytes_read < 0 {
                    break;
                }
            }
        });
        
        self.rumble_thread = Some(handle);
    }
}