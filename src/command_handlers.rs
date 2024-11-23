use rumqttc::{AsyncClient, QoS};
use serde_json::{json, Value};
use system_shutdown::shutdown;
use windows::{
    Media::Control::GlobalSystemMediaTransportControlsSessionManager,
    Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties,
    Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Foundation::IAsyncOperation;
use windows::Media::Control::GlobalSystemMediaTransportControlsSession;
use windows_volume_control::AudioController;

use crate::{DEVICE_ID, USER_ID};

pub(crate) async unsafe fn set_volume(data: &Value, client: &AsyncClient) {
    let mut controller = AudioController::init(None);
    controller.GetSessions();
    controller.GetDefaultAudioEnpointVolumeControl();
    controller.GetAllProcessSessions();
    controller.GetSessions();
    let master_session = controller.get_session_by_name("master".to_string());
    match data["params"]["volumeLevel"].as_f64() {
        None => {}
        Some(val) => {
            master_session.unwrap().setVolume((val / 100.0) as f32);
            send_status(client).await;
        }
    }
}


pub(crate) async unsafe fn mute(data: &Value, client: &AsyncClient) {
    let mut controller = AudioController::init(None);
    controller.GetSessions();
    controller.GetDefaultAudioEnpointVolumeControl();
    controller.GetAllProcessSessions();
    let master_session = controller.get_session_by_name("master".to_string());
    match data["params"]["mute"].as_bool() {
        None => {}
        Some(val) => {
            master_session.unwrap().setMute(val);
            send_status(client).await;
        }
    }
}

pub(crate) async fn set_volume_relative(data: Value, client: &AsyncClient) {
    unsafe {
        let mut controller = AudioController::init(None);
        controller.GetSessions();
        controller.GetDefaultAudioEnpointVolumeControl();
        controller.GetAllProcessSessions();
        let master_session = controller.get_session_by_name("master".to_string());
        match data["params"]["relativeSteps"].as_f64() {
            None => {}
            Some(val) => {
                let ms = master_session.unwrap();
                ms.setVolume((ms.getVolume() + (val / 100.0) as f32).max(0.0).min(1.0));
                send_status(client).await;
            }
        }
    }
}


async fn get_media_session_manager() -> Option<GlobalSystemMediaTransportControlsSessionManager> {
    // Attempt to get a media session manager asynchronously
    match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
        Ok(operation) => operation.get().ok(),  // Await the async operation and handle errors
        Err(_) => None,  // Return None if the RequestAsync call itself fails
    }
}

pub(crate) async unsafe fn pause(data: Value, client: &AsyncClient) {
    if let Some(manager) = get_media_session_manager().await {
        if let Some(session) = manager.GetCurrentSession().ok() {
            session.TryPauseAsync().ok();
            send_status(client).await;
        } else {
            log::debug!("Could not find current session ...");
        }
    } else {
        log::debug!("Could not find media session manager ...");
    }
}

pub(crate) async unsafe fn unpause(data: Value, client: &AsyncClient) {
    if let Some(manager) = get_media_session_manager().await {
        if let Some(session) = manager.GetCurrentSession().ok() {
            session.TryPlayAsync().ok();
            send_status(client).await;
        }
    }
}

pub(crate) async unsafe fn next_track(data: Value, client: &AsyncClient) {
    if let Some(manager) = get_media_session_manager().await {
        if let Some(session) = manager.GetCurrentSession().ok() {
            session.TrySkipNextAsync().ok();
            send_status(client).await;
        }
    }
}

pub(crate) async unsafe fn previous_track(data: Value, client: &AsyncClient) {
    if let Some(manager) = get_media_session_manager().await {
        if let Some(session) = manager.GetCurrentSession().ok() {
            session.TrySkipPreviousAsync().ok();
            send_status(client).await;
        }
    }
}

async fn get_playingstate() -> Option<GlobalSystemMediaTransportControlsSessionPlaybackStatus> {
    if let Some(manager) = get_media_session_manager().await {
        if let Some(session) = manager.GetCurrentSession().ok() {
            if let Ok(playback_info) = session.GetPlaybackInfo() {
                return playback_info.PlaybackStatus().ok();
            }
        }
    }
    None
}

pub(crate) async fn on_off(data: Value, client: &AsyncClient) {
    let on_off = data["params"]["on"].as_bool().expect("On of is required");
    if !on_off {
        match shutdown() {
            Ok(_) => println!("Shutting down, bye!"),
            Err(error) => eprintln!("Failed to shut down: {}", error),
        }
    }
}

async fn get_media_info() -> Option<(String, String, String)> {
    if let Some(manager) = get_media_session_manager().await {
        if let Some(session) = manager.GetCurrentSession().ok() {
            if let Ok(properties_op) = session.TryGetMediaPropertiesAsync() {
                if let Ok(properties) = properties_op.get() {
                    let title = properties.Title().unwrap_or_default();
                    let artist = properties.Artist().unwrap_or_default();
                    let album = properties.AlbumTitle().unwrap_or_default();
                    return Some((title.to_string(), artist.to_string(), album.to_string()));
                }
            }
        }
    }
    None
}

pub(crate) async unsafe fn send_status(client: &AsyncClient) {
    let mut controller = AudioController::init(None);
    controller.GetSessions();
    controller.GetDefaultAudioEnpointVolumeControl();
    controller.GetAllProcessSessions();
    let master_session = controller.get_session_by_name("master".to_string());


    let mut status = json!({
        "currentVolume": master_session.unwrap().getVolume() * 100.0,
        "isMuted": master_session.unwrap().getMute(),
        "on": true
    });
    if let Some((title, artist, album)) = get_media_info().await {
        status["playbackState"] = Value::String(match get_playingstate().await {
            None => "STOPPED".to_string(),
            Some(state) => {
                match state {
                    GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => "PLAYING".to_string(),
                    GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => "PAUSED".to_string(),
                    _ => "STOPPED".to_string(),
                }
            }
        })
    }


    client.publish(format!("/ada/{USER_ID}/{DEVICE_ID}/status"), QoS::AtLeastOnce, false, status.to_string()).await.unwrap()
}


