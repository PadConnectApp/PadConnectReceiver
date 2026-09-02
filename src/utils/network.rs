/*
 * Copyright (C) 2026 Ishan
 *
 * This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, version 3 only.
 *
 * This program is distributed without any warranty. See the GNU General Public License for more details.
 */

use std::net::{UdpSocket, SocketAddr};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;
use crate::data::GamepadState;

pub const MIN_SUPPORTED_VERSION: i32 = 2;
pub const FEATURE_RUMBLE: i32 = 1 << 0;
pub const FEATURE_LATENCY: i32 = 1 << 1;

pub struct DiscoveryServer {
    port: u16,
    is_running: Arc<AtomicBool>,
}

impl DiscoveryServer {
    pub fn new(port: u16) -> Self {
        Self { port, is_running: Arc::new(AtomicBool::new(false)) }
    }

    pub fn start(&self, on_responded: impl Fn(i32) + Send + 'static) {
        if self.is_running.swap(true, Ordering::SeqCst) { return; }
        
        let socket = UdpSocket::bind(("0.0.0.0", self.port)).unwrap();
        socket.set_read_timeout(Some(std::time::Duration::from_millis(500))).unwrap();
        let running = Arc::clone(&self.is_running);

        thread::spawn(move || {
            let mut buf = [0u8; 256];
            while running.load(Ordering::SeqCst) {
                if let Ok((amt, src)) = socket.recv_from(&mut buf) {
                    let msg = String::from_utf8_lossy(&buf[..amt]);
                    if msg.starts_with("PADCONNECT_DISCOVER") {
                        let parts: Vec<&str> = msg.split(':').collect();
                        let client_version = parts.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
                        let client_features = parts.get(2).and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);

                        if client_version < MIN_SUPPORTED_VERSION {
                            println!("App Update Required for client.");
                        }

                        let agreed_version = client_version.min(MIN_SUPPORTED_VERSION);
                        let agreed_features = client_features & (FEATURE_RUMBLE | FEATURE_LATENCY);
                        
                        let response = format!("PADCONNECT_HERE:8082:{}:{}", agreed_version, agreed_features);
                        let _ = socket.send_to(response.as_bytes(), src);
                        
                        on_responded(agreed_features);
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}

pub struct UdpReceiver {
    port: u16,
    is_running: Arc<AtomicBool>,
    is_latency_enabled: Arc<AtomicBool>,
    is_rumble_enabled: Arc<AtomicBool>,
    pub current_sender: Arc<Mutex<Option<SocketAddr>>>,
    socket: Arc<Mutex<Option<UdpSocket>>>,
}

impl UdpReceiver {
    pub fn new(port: u16) -> Self {
        Self { 
            port, 
            is_running: Arc::new(AtomicBool::new(false)),
            is_latency_enabled: Arc::new(AtomicBool::new(false)),
            is_rumble_enabled: Arc::new(AtomicBool::new(false)),
            current_sender: Arc::new(Mutex::new(None)),
            socket: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_enabled_features(&self, features: i32) {
        self.is_rumble_enabled.store((features & FEATURE_RUMBLE) != 0, Ordering::SeqCst);
        self.is_latency_enabled.store((features & FEATURE_LATENCY) != 0, Ordering::SeqCst);
    }

    pub fn start(&self, on_event: impl Fn(GamepadState) + Send + 'static) {
        if self.is_running.swap(true, Ordering::SeqCst) { return; }
        
        let socket = UdpSocket::bind(("0.0.0.0", self.port)).unwrap();
        *self.socket.lock().unwrap() = Some(socket.try_clone().expect("Failed to clone socket"));
        let running = Arc::clone(&self.is_running);
        let sender_ref = Arc::clone(&self.current_sender);
        let latency_enabled = Arc::clone(&self.is_latency_enabled);

        thread::spawn(move || {
            let mut buf = [0u8; 21];
            while running.load(Ordering::SeqCst) {
                if let Ok((amt, src)) = socket.recv_from(&mut buf) {
                    let mut cursor = Cursor::new(&buf[..amt]);
                    if let Ok(packet_type) = cursor.read_u8() {
                        if packet_type == 0 {
                            let state = GamepadState {
                                buttons: cursor.read_u16::<LittleEndian>().unwrap_or(0),
                                lx: cursor.read_i16::<LittleEndian>().unwrap_or(0),
                                ly: cursor.read_i16::<LittleEndian>().unwrap_or(0),
                                rx: cursor.read_i16::<LittleEndian>().unwrap_or(0),
                                ry: cursor.read_i16::<LittleEndian>().unwrap_or(0),
                                lt: cursor.read_u8().unwrap_or(0),
                                rt: cursor.read_u8().unwrap_or(0),
                            };

                            *sender_ref.lock().unwrap() = Some(src);
                            on_event(state);

                            if latency_enabled.load(Ordering::SeqCst) {
                                if let Ok(sent_time) = cursor.read_i64::<LittleEndian>() {
                                    Self::send_latency(&socket, src, sent_time);
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    fn send_latency(socket: &UdpSocket, target: SocketAddr, sent_time: i64) {
        let mut response = Vec::with_capacity(17);
        response.push(2u8); // Packet Type 2

        let now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        let _ = response.write_i64::<LittleEndian>(sent_time);
        let _ = response.write_i64::<LittleEndian>(now_nanos);

        let _ = socket.send_to(&response, target);
    }

    pub fn on_rumble(&self, large: u8, small: u8) {
        if !self.is_rumble_enabled.load(Ordering::SeqCst) {
            return;
        }

        if let Some(target) = *self.current_sender.lock().unwrap() {
            if let Some(socket) = self.socket.lock().unwrap().as_ref() {
                let packet = [1u8, large, small];
                let _ = socket.send_to(&packet, target);
            }
        }
    }
    
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}