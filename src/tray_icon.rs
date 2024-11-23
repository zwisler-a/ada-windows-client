use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use std::thread;
use tray_item::{IconSource, TrayItem};

pub enum Message {
    Quit
}

pub(crate) fn define_tray_menu() -> Receiver<Message> {
    log::debug!("Defining tray icon");

    let (tx, rx) = mpsc::channel(1);

    thread::spawn(move || {
        let mut tray = TrayItem::new(
            "Ada",
            IconSource::Resource("ada-icon"),
        )
            .unwrap();
        tray.add_label("Ada desktop client").unwrap();

        let quit_tx = tx.clone();
        tray.add_menu_item("Quit", move || {
            quit_tx.blocking_send(Message::Quit).unwrap();
        })
            .unwrap();

        loop {
            std::thread::park();
        }
    });

    rx
}