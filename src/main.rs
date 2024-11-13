#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate image;

mod authentification;
mod connectivity;
mod gui_command;
mod playback_controls;
use gui_command::{GuiApp, GuiCommand};
use image::EncodableLayout;
use std::time::Duration;

use tokio::sync::mpsc;

use connectivity::connect_to_device;

use slint::{Image, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel};

slint::include_modules!();

#[tokio::main]
async fn main() {
    // Setup
    env_logger::init();

    let ui = AppWindow::new().unwrap();
    let ui_handle = ui.as_weak();
    let ui_handle_2 = ui_handle.clone();
    let ui_handle_3 = ui_handle.clone();

    let (tx1, mut rx1) = mpsc::channel::<GuiCommand>(32);

    ///// Callbacks //////

    // Handle Play/Pause Callback
    let play_pause_tx = tx1.clone();
    ui.on_play_pause(move || {
        let use_tx1 = play_pause_tx.clone();
        tokio::task::block_in_place(move || {
            use_tx1
                .blocking_send(GuiCommand::PlayPause)
                .expect("Couldn't Play/Pause");
        })
    });

    // Handle Seeking
    let click_tx = tx1.clone();
    ui.on_seek(move || {
        let ui2 = ui_handle_2.clone();
        let use_tx1 = click_tx.clone();
        let ui3 = ui2.unwrap();

        tokio::task::block_in_place(move || {
            use_tx1
                .blocking_send(GuiCommand::Seek(ui3.get_x_location()))
                .expect("Couldn't Seek");
        });
    });

    // Easter Egg
    ui.on_change_debug(move || {
        let ui2 = ui_handle_3.clone();
        let ui3 = ui2.unwrap();
        ui3.set_debug_icons(!ui3.get_debug_icons());
    });

    // Handle Shuffle Callback
    let shuffle_tx = tx1.clone();
    ui.on_shuffle(move || {
        let use_tx1 = shuffle_tx.clone();
        tokio::task::block_in_place(move || {
            use_tx1
                .blocking_send(GuiCommand::Shuffle)
                .expect("Couldn't Shuffle");
        });
    });

    // Handle App Tray Callback
    let app_tx = tx1.clone();
    ui.on_show_tray(move || {
        let use_tx1 = app_tx.clone();
        tokio::task::block_in_place(move || {
            use_tx1
                .blocking_send(GuiCommand::AppTray)
                .expect("Couldn't Open App List");
        });
    });

    // Handle App Response Callback
    let bt_tx = tx1.clone();
    ui.on_select_app(move || {
        let use_tx1 = bt_tx.clone();
        tokio::task::block_in_place(move || {
            let strings = (1..20)
                .map(|x| format!("Device {}", x))
                .collect::<Vec<String>>();
            use_tx1
                .blocking_send(GuiCommand::AppSelect(vec![
                    GuiApp::Music {
                        name: "Music".into(),
                    },
                    GuiApp::Bluetooth {
                        name: "Bluetooth".into(),
                        data: strings,
                    },
                ]))
                .expect("Couldn't Select App");
        });
    });

    // Handle Bluetooth Receive Callback
    let bt2_tx = tx1.clone();
    ui.on_select_device(move || {
        let use_tx1 = bt2_tx.clone();
        tokio::task::block_in_place(move || {
            connect_to_device("".into());
            use_tx1
                .blocking_send(GuiCommand::AppTray)
                .expect("Couldn't Open App List after Bluetooth");
        });
    });

    let like_tx = tx1.clone();
    ui.on_like(move || {
        let use_tx1 = like_tx.clone();
        tokio::task::block_in_place(move || {
            use_tx1
                .blocking_send(GuiCommand::Like)
                .expect("Couldn't Like song");
        });
    });

    let backward_tx = tx1.clone();
    ui.on_skip_backward(move || {
        let use_tx1 = backward_tx.clone();
        tokio::task::block_in_place(move || {
            use_tx1
                .blocking_send(GuiCommand::Back)
                .expect("Going to previous song didn't work");
        });
    });

    let forward_tx = tx1.clone();
    ui.on_skip_forward(move || {
        let use_tx1 = forward_tx.clone();
        tokio::task::block_in_place(move || {
            use_tx1
                .blocking_send(GuiCommand::Forward)
                .expect("Going to next song didn't work");
        });
    });

    ///// Update loops //////

    // Update everything periodically
    let update_tx = tx1.clone();
    tokio::task::spawn(async move {
        loop {
            // update_tx
            //     .send("Deep Update".into())
            //     .await
            //     .expect("Couldn't update song");
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });

    // Update progress bar periodically
    let time_tx = tx1.clone();
    tokio::task::spawn(async move {
        loop {
            time_tx
                .send(GuiCommand::UpdateProgress)
                .await
                .expect("Couldn't update progress bar");
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    ///// Thread for media controls //////

    let _tokio_thread = tokio::spawn(async move {
        //let spotify = setup_spotify().await;
        //let mut state = get_state_from_spotify(&spotify).await;

        //let (mut queue, mut loc_in_queue) = get_buffer_and_location(&spotify).await;
        loop {
            tokio::select! {
                val = rx1.recv() => {

                    match val.expect("Error receiving Gui command") {


                        GuiCommand::AppSelect(apps) => {
                            let ui2 = ui_handle.clone();
                            slint::invoke_from_event_loop(move || {
                                let ui = ui2.unwrap();

                                let app_name = ui.get_selected_app();

                                apps.into_iter().for_each(|app| match app {
                                    GuiApp::Music { name } => {
                                        if app_name == name {
                                            ui.set_display_bluetooth(false);
                                            ui.set_display_apps(false);
                                        }
                                    }
                                    GuiApp::Bluetooth { name, data } => {
                                        if app_name == name {
                                            let shared_strings = data.into_iter().map(SharedString::from).collect::<Vec<_>>();
                                            let model = VecModel::from(shared_strings);
                                            let devices_model = slint::ModelRc::new(model);
                                            ui.set_bluetooth_devices(devices_model);
                                            ui.set_display_bluetooth(true);
                                            ui.set_display_apps(false);
                                        }
                                    }
                                });

                             }).unwrap();
                        }

                        GuiCommand::AppTray => {
                            let ui2 = ui_handle.clone();
                            println!("App Tray");
                            slint::invoke_from_event_loop(move || {
                                let ui = ui2.unwrap();
                                ui.set_display_bluetooth(false);
                                ui.set_display_apps(true);
                             }).unwrap();
                        }


                        GuiCommand::PlayPause => {
                                // let _ = change_playback_state(&spotify, &mut state).await;
                                // guess_current_progress(&spotify, &mut state).await;
                                // let ui2 = ui_handle.clone();
                                // slint::invoke_from_event_loop(move || {
                                //     let ui = ui2.unwrap();
                                //     ui.set_paused(state.is_playing);

                                //  }).unwrap();

                        },

                        GuiCommand::Shuffle => {
                            // let _ = shuffle(&spotify, &mut state).await;
                            // let ui2 = ui_handle.clone();
                            //     slint::invoke_from_event_loop(move || {
                            //         let ui = ui2.unwrap();
                            //         ui.set_shuffled(state.shuffle);

                            //      }).unwrap();
                        },

                        GuiCommand::Like => {
                            // let _ = like(&spotify, &mut state).await;
                            // let ui2 = ui_handle.clone();
                            //     slint::invoke_from_event_loop(move || {
                            //         let ui = ui2.unwrap();
                            //         ui.set_liked(state.liked);
                            //      }).unwrap();
                        },

                        GuiCommand::Back => {
                            // let _ = skip_backward(&spotify, &mut state).await;
                            // if loc_in_queue > 0 {
                            //     loc_in_queue -= 1;

                            //     weak_update_state_item(&spotify, &mut state, &queue, &mut loc_in_queue).await;
                            // } else {
                            //     update_state_item(&spotify, &mut state).await;
                            // }
                        },

                        GuiCommand::Forward => {
                            // let _ = skip_forward(&spotify, &mut state).await;
                            // if loc_in_queue < queue.len() - 1 {

                            //     weak_update_state_item(&spotify, &mut state, &queue, &mut loc_in_queue).await;
                            //     loc_in_queue += 1;

                            // }
                        },

                        GuiCommand::RefreshUI{title, artist_name, album_name, album_art, duration, is_playing, is_liked, is_shuffle} => {
                            if let Some(title) = title {
                                let ui2 = ui_handle.clone();
                                slint::invoke_from_event_loop(move || {
                                    let ui = ui2.unwrap();
                                    ui.set_song_name(SharedString::from(title));
                                 }).unwrap();
                            }
                            if let Some(artist_name) = artist_name {
                                let ui2 = ui_handle.clone();
                                slint::invoke_from_event_loop(move || {
                                    let ui = ui2.unwrap();
                                    ui.set_artist_name(SharedString::from(artist_name));
                                 }).unwrap();
                            }
                            if let Some(album_name) = album_name {
                                let ui2 = ui_handle.clone();
                                slint::invoke_from_event_loop(move || {
                                    let ui = ui2.unwrap();
                                    ui.set_album_name(SharedString::from(album_name));
                                 }).unwrap();
                            }
                            if let Some(image_bytes) = album_art {
                                let ui2 = ui_handle.clone();
                                slint::invoke_from_event_loop(move || {
                                    let img = image::load_from_memory(&image_bytes).unwrap().into_rgba8();
                                    let color = color_thief::get_palette(img.as_bytes(), color_thief::ColorFormat::Rgba, 5, 2).unwrap()[0];
                                    let use_color = slint::Color::from_rgb_u8(color.r, color.g, color.b);
                                    let whiten = (color.r as f32 * color.r as f32 + color.g as f32 * color.g as f32 + color.b as f32 * color.b as f32).sqrt() <= 120.0;
                                    let shared_buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                                        img.as_raw(),
                                        img.width(),
                                        img.height(),
                                    );
                                    let image = Image::from_rgba8(shared_buf);
                                    let ui = ui2.unwrap();
                                    ui.set_background_color(use_color);
                                    ui.set_song_image(image);
                                    ui.set_use_white_text(whiten);
                                 }).unwrap();
                            }
                            if let Some(duration) = duration {
                                let ui2 = ui_handle.clone();
                                slint::invoke_from_event_loop(move || {
                                    let ui = ui2.unwrap();
                                    ui.set_time(duration);
                                 }).unwrap();
                            }
                            if let Some(is_playing) = is_playing {
                                let ui2 = ui_handle.clone();
                                slint::invoke_from_event_loop(move || {
                                    let ui = ui2.unwrap();
                                    ui.set_paused(!is_playing);
                                 }).unwrap();
                            }
                            if let Some(is_liked) = is_liked {
                                let ui2 = ui_handle.clone();
                                slint::invoke_from_event_loop(move || {
                                    let ui = ui2.unwrap();
                                    ui.set_liked(is_liked);
                                 }).unwrap();
                            }
                            if let Some(is_shuffle) = is_shuffle {
                                let ui2 = ui_handle.clone();
                                slint::invoke_from_event_loop(move || {
                                    let ui = ui2.unwrap();
                                    ui.set_shuffled(is_shuffle);
                                 }).unwrap();
                            }
                        },

                        GuiCommand::Seek(num) => {

                            // state.seek_time = num;
                            // let _ = seek_in_track(&spotify, &mut state).await;
                            // let ui2 = ui_handle.clone();
                            // slint::invoke_from_event_loop(move || {
                            //     let ui = ui2.unwrap();
                            //     ui.set_time(state.percentage);
                            // }).unwrap();

                        },

                        GuiCommand::UpdateProgress => {
                            let ui2 = ui_handle.clone();
                            slint::invoke_from_event_loop(move || {
                                let ui = ui2.unwrap();
                                ui.set_time(ui.get_time() + 0.1);
                            }).unwrap();
                        },

                        _ => {
                            println!("Unhandled command");
                        },
                    }
                }
            }
        }
    });

    // Create the window
    ui.run().unwrap();
}
