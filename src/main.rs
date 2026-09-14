/*
 * Copyright (C) 2026 Ishan
 *
 * This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, version 3 only.
 *
 * This program is distributed without any warranty. See the GNU General Public License for more details.
 */

mod data;
mod input;
mod utils;
mod viewmodel;

use slint::ComponentHandle;
use viewmodel::ReceiverViewModel;

slint::slint! {
    import { VerticalBox, HorizontalBox } from "std-widgets.slint";

    component DataCard inherits Rectangle {
        in property <string> title;
        in property <string> value;

        background: #2a2a2a;
        border-radius: 8px;
        border-width: 1px;
        border-color: #3d3d3d;

        VerticalBox {
            padding: 16px;
            spacing: 8px;
            Text {
                text: root.title;
                font-size: 12px;
                color: #9ca3af;
            }
            Text {
                text: root.value;
                font-size: 16px;
                color: #ffffff;
                font-weight: 600;
            }
        }
    }

    export component ReceiverScreen inherits Window {
        title: "PadConnectReceiver";
        icon: @image-url("icons/icon.ico");
        preferred-width: 480px;
        preferred-height: 380px;
        background: #1a1a1a;

        in property <string> connection-status: "Searching...";
        in property <color> status-color: #3b82f6;
        
        in property <color> dialog-color: #3b82f6;

        in property <string> btn-state: "-";
        in property <string> l-stick: "-";
        in property <string> r-stick: "-";
        in property <string> triggers: "-";

        in property <string> alert_message: "";
        callback show_alert();

        alert_popup := PopupWindow {
            x: (root.width - alert_popup.width) / 2;
            y: (root.height - alert_popup.height) / 2;
            width: 320px;

            Rectangle {
                background: #242424;
                border-color: #333333;
                border-width: 1px;
                border-radius: 8px;

                drop-shadow-color: black.transparentize(40%);
                drop-shadow-blur: 24px;
                drop-shadow-offset-y: 8px;

                VerticalBox {
                    padding: 24px;
                    spacing: 16px;

                    Text {
                        text: "Alert";
                        font-size: 16px;
                        font-weight: 600;
                        color: white;
                    }

                    Text {
                        text: root.alert_message;
                        wrap: word-wrap;
                        color: #cccccc;
                        font-size: 14px;
                        min-height: 40px;
                    }

                    HorizontalBox {
                        padding: 0px;
                        alignment: end;

                        Rectangle {
                            width: 80px;
                            height: 32px;
                            border-radius: 4px;
                            
                            background: dialog_touch.pressed ? root.dialog-color.darker(20%) : 
                                       (dialog_touch.has-hover ? root.dialog-color.brighter(10%) : root.dialog-color);

                            dialog_touch := TouchArea {
                                clicked => { alert_popup.close(); }
                            }

                            Text {
                                text: "Close";
                                color: white;
                                font-size: 14px;
                                font-weight: 600;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                        }
                    }
                }
            }
        }

        show_alert => {
            alert_popup.show();
        }

        VerticalBox {
            padding: 24px;
            spacing: 16px;

            Rectangle {
                background: #2a2a2a;
                border-radius: 12px;
                border-width: 1px;
                border-color: #3d3d3d;
                height: 70px;

                HorizontalBox {
                    padding: 16px;
                    alignment: space-between;

                    Text {
                        text: "Status";
                        font-size: 18px;
                        color: #ffffff;
                        font-weight: 600;
                        vertical-alignment: center;
                    }

                    Rectangle {
                        background: root.status-color.transparentize(85%);
                        border-radius: 16px;
                        border-width: 1px;
                        border-color: root.status-color;

                        HorizontalBox {
                            padding-left: 16px;
                            padding-right: 16px;
                            Text {
                                text: root.connection-status;
                                color: root.status-color;
                                font-size: 13px;
                                font-weight: 700;
                                vertical-alignment: center;
                            }
                        }
                    }
                }
            }

            HorizontalBox {
                spacing: 16px;
                padding: 0px;
                DataCard { title: "Buttons (Bitmask)"; value: root.btn-state; }
                DataCard { title: "Triggers (L / R)"; value: root.triggers; }
            }
            HorizontalBox {
                spacing: 16px;
                padding: 0px;
                DataCard { title: "Left Stick (X / Y)"; value: root.l-stick; }
                DataCard { title: "Right Stick (X / Y)"; value: root.r-stick; }
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    // set to trace level as we already do compile time filtering so we want to see all logs in the console
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let ui = ReceiverScreen::new()?;
    let ui_handle = ui.as_weak();

    let view_model = ReceiverViewModel::new(
        {
            let ui_handle = ui_handle.clone();

            move |state, is_connected| {
                let ui_handle = ui_handle.clone();

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        if is_connected {
                            ui.set_connection_status("Connected".into());
                            ui.set_status_color(slint::Color::from_rgb_u8(34, 197, 94));

                            ui.set_btn_state(format!("{}", state.buttons).into());
                            ui.set_l_stick(format!("{} / {}", state.lx, state.ly).into());
                            ui.set_r_stick(format!("{} / {}", state.rx, state.ry).into());
                            ui.set_triggers(format!("{} / {}", state.lt, state.rt).into());
                        } else {
                            ui.set_connection_status("Searching...".into());
                            ui.set_status_color(slint::Color::from_rgb_u8(59, 130, 246));

                            ui.set_btn_state("-".into());
                            ui.set_l_stick("-".into());
                            ui.set_r_stick("-".into());
                            ui.set_triggers("-".into());
                        }
                    }
                });
            }
        },
        {
            move |alert_msg: String| {
                let ui_handle = ui_handle.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_alert_message(alert_msg.into());
                        ui.invoke_show_alert();
                    }
                });
            }
        }
    );

    ui.run()?;
    view_model.shutdown();
    Ok(())
}
