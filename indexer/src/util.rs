use firedbg_rust_debugger::{DebuggerInfo, InfoMessage};
use sea_streamer::{Buffer, Message, SharedMessage};

/// Deserialize a message payload from JSON.
///
/// # Panics
///
/// Panics if deserialization fails.
pub fn deser<T: serde::de::DeserializeOwned>(m: &SharedMessage) -> T {
    match try_deser(m) {
        Ok(v) => v,
        Err(e) => panic!("Deserialization failed: {e}"),
    }
}

fn try_deser<T: serde::de::DeserializeOwned>(m: &SharedMessage) -> Result<T, String> {
    m.message().deserialize_json().map_err(|e| {
        format!(
            "Failed to deserialize message `{}`: {}",
            m.message().as_str().unwrap_or("<non-utf8 message>"),
            e
        )
    })
}

pub fn deser_info(m: &SharedMessage) -> InfoMessage {
    if let Ok(info) = try_deser::<InfoMessage>(m) {
        info
    } else {
        let info = deser::<DebuggerInfo>(m);
        InfoMessage::Debugger(info)
    }
}
