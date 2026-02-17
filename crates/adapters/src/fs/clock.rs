use lite_room_application::Clock;

#[derive(Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_timestamp_string(&self) -> String {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        secs.to_string()
    }
}
