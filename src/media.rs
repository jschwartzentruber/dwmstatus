use crate::prelude::*;
use mpris::{PlaybackStatus, PlayerFinder};

static ICON: &str = "▶";

pub fn status() -> Option<String> {
    match PlayerFinder::new() {
        Ok(player_finder) => {
            match player_finder.find_active() {
                Ok(player) => {
                    match player.get_metadata() {
                        Ok(metadata) => {
                            let title = {
                                let _title = metadata.title().unwrap_or("");
                                if _title.len() > 0 {
                                    _title
                                } else {
                                    "?"
                                }
                            };
                            let track = match metadata.artists() {
                                Some(v) => {
                                    // YT Music seems to append '- Topic' to every artist
                                    let artists = str::replace(&v.join(" & "), " - Topic", "");
                                    if artists.len() > 0 {
                                        Some(artists + " - ")
                                    } else {
                                        None
                                    }
                                },
                                None => None,
                            }.unwrap_or("".to_string()) + title;
                            match player.get_playback_status() {
                                Ok(status) => match status {
                                    PlaybackStatus::Playing => Some(GOOD.to_string() + ICON + " " + &track),
                                    PlaybackStatus::Paused => Some(WARN.to_string() + ICON + " " + &track),
                                    PlaybackStatus::Stopped => None,
                                },
                                Err(_e) => None,
                            }
                        },
                        Err(_e) => None,  // no metadata for player
                    }
                },
                Err(_e) => None,  // no player
            }
        },
        Err(_e) => None,  // no D-Bus
    }
}
