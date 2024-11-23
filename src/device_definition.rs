use serde_json::{json, Value};

pub(crate) fn get_device_info() -> Value {
    return json!({
          "id": "<overwritten>",
          "type": "action.devices.types.REMOTECONTROL",
          "traits": [
            "action.devices.traits.OnOff",
            "action.devices.traits.Volume",
            "action.devices.traits.TransportControl",
            "action.devices.traits.MediaState"
          ],
          "name": {
            "name": "<overwritten>"
          },
          "willReportState": true,
          "attributes": {
            "supportPlaybackState": true,
            "volumeMaxLevel": 100,
            "volumeCanMuteAndUnmute": true,
            "levelStepSize": 1,
            "commandOnlyVolume": false,
            "volumeDefaultPercentage": 36,
            "transportControlSupportedCommands": [
              "NEXT",
              "PREVIOUS",
              "PAUSE",
              "STOP",
              "RESUME"
            ]
          },
          "deviceInfo": {
            "manufacturer": "Zwisler",
            "model": "rust-pc-control",
            "hwVersion": "1.0",
            "swVersion": "1.0"
          }
        }
    );
}