/*
 * Copyright (C) 2026 Ishan
 *
 * This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, version 3 only.
 *
 * This program is distributed without any warranty. See the GNU General Public License for more details.
 */

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::time::Duration;

use crate::input::xinput::InputExecutor;
#[cfg(target_os = "windows")]
use crate::input::xinput::XInputExecutor;

#[cfg(not(target_os = "windows"))]
use crate::input::uinput::UinputExecutor;

use crate::utils::network::{DiscoveryServer, UdpReceiver};
use crate::data::GamepadState;

use log::{info};

pub struct ReceiverViewModel {
    discovery: Arc<DiscoveryServer>,
    receiver: Arc<UdpReceiver>,
    is_receiving: Arc<AtomicBool>,
    last_receive_time: Arc<AtomicU64>,
}

impl ReceiverViewModel {
    pub fn new<F1, F2>(on_ui_update: F1, on_alert: F2) -> Self 
    where 
        F1: Fn(GamepadState, bool) + Send + Sync + 'static,
        F2: Fn(String) + Send + Sync + 'static,
    {
        let on_ui_update = Arc::new(on_ui_update);
        let on_alert = Arc::new(on_alert);

        let discovery = Arc::new(DiscoveryServer::new(8083));
        let receiver = Arc::new(UdpReceiver::new(8082));

        let receiver_clone = Arc::clone(&receiver);
        
        #[cfg(target_os = "windows")]
        let executor: Arc<Mutex<Box<dyn InputExecutor>>> = Arc::new(Mutex::new(
            Box::new(XInputExecutor::new().expect("ViGEm failed"))
        ));

        #[cfg(not(target_os = "windows"))]
        let executor: Arc<Mutex<Box<dyn InputExecutor>>> = Arc::new(Mutex::new(
            Box::new(UinputExecutor::new().expect("Failed to create uinput device"))
        ));

        let receiver_rumble = Arc::clone(&receiver);
        if let Ok(mut guard) = executor.lock() {
            guard.set_rumble_callback(Box::new(move |large, small| {
                receiver_rumble.on_rumble(large, small);
            }));
        }

        let is_receiving = Arc::new(AtomicBool::new(false));
        let last_receive_time = Arc::new(AtomicU64::new(0));

        let is_rec_clone = Arc::clone(&is_receiving);
        let time_clone = Arc::clone(&last_receive_time);
        let disc_clone = Arc::clone(&discovery);
        let ui_cb = Arc::clone(&on_ui_update);
        let on_alert_cb = Arc::clone(&on_alert);

        let executor_clone = Arc::clone(&executor);

        receiver.start(move |state| {
            time_clone.store(
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64, 
                Ordering::SeqCst
            );

            if !is_rec_clone.load(Ordering::SeqCst) {
                is_rec_clone.store(true, Ordering::SeqCst);
                disc_clone.stop();
                info!("PadConnect connected!");
            }

            if let Ok(mut guard) = executor_clone.lock() {
                guard.submit(&state);
            }
            
            ui_cb(state, true);
        });

        discovery.start(
            move |features| {
                receiver_clone.set_enabled_features(features);
                info!("Discovery Agreed features: {}", features);
            },
            move |alert_msg| {
                on_alert_cb(alert_msg); 
            }
        );

        let vm = Self { discovery, receiver, is_receiving, last_receive_time };
        vm.start_connection_monitor(Arc::clone(&on_ui_update), Arc::clone(&on_alert));
        vm
    }

    fn start_connection_monitor<F1, F2>(&self, on_ui_update: Arc<F1>, on_alert: Arc<F2>) 
    where 
        F1: Fn(GamepadState, bool) + Send + Sync + 'static,
        F2: Fn(String) + Send + Sync + 'static,
    {
        let is_receiving = Arc::clone(&self.is_receiving);
        let last_time = Arc::clone(&self.last_receive_time);
        let discovery = Arc::clone(&self.discovery);
        let receiver_clone = Arc::clone(&self.receiver);

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(500));
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                let last = last_time.load(Ordering::SeqCst);

                if is_receiving.load(Ordering::SeqCst) && (current_time.saturating_sub(last) > 2000) {
                    is_receiving.store(false, Ordering::SeqCst);
                    info!("PadConnect disconnected. Restarting discovery server.");
                    on_ui_update(GamepadState::default(), false);
                    let rec_feat = Arc::clone(&receiver_clone);
                    let on_alert_cb = Arc::clone(&on_alert);
                    discovery.start(
                        move |features| {
                            rec_feat.set_enabled_features(features);
                            info!("Discovery agreed features: {}", features);
                        },
                        move |alert_msg| {
                            on_alert_cb(alert_msg);
                        }
                    );
                }
            }
        });
    }

    pub fn shutdown(&self) {
        self.receiver.stop();
        self.discovery.stop();
    }
}