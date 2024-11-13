use std::time::Instant;

use rspotify::{
    model::{Id, TrackId},
    prelude::{BaseClient, OAuthClient},
    scopes, AuthCodeSpotify, Config, Credentials, OAuth,
};

use crate::{
    playback_controls::State,
    playback_controls::{get_liked_state, get_shuffle_state, update_time},
};

pub async fn setup_spotify() -> AuthCodeSpotify {
    let creds = Credentials::from_env().unwrap();

    let oauth = OAuth::from_env(scopes!(
        "user-read-playback-state",
        "user-modify-playback-state",
        "user-read-currently-playing",
        "app-remote-control",
        "playlist-read-private",
        "playlist-read-collaborative",
        "playlist-modify-private",
        "playlist-modify-public",
        "user-read-playback-position",
        "user-read-recently-played",
        "user-library-modify",
        "user-library-read"
    ))
    .unwrap();

    let config_dir = dirs::config_dir().expect("you don't have a config dir wtf");
    let cache_path = config_dir.join("car_thang");

    let _ = std::fs::create_dir_all(&cache_path);

    let spotify = AuthCodeSpotify::with_config(
        creds,
        oauth,
        Config {
            token_cached: true,
            cache_path: cache_path.join("hawk_creds.json"),
            ..Default::default()
        },
    );

    let url = spotify.get_authorize_url(false).unwrap();

    spotify.prompt_for_token(&url).await.unwrap();
    spotify
}

pub async fn get_state_from_spotify(spotify: &AuthCodeSpotify) -> State {
    let current = spotify.current_playing(None, None::<Vec<_>>).await;

    let use_current = current.unwrap().unwrap().clone();

    let is_playing = use_current.is_playing;
    let progress_ms = use_current.progress.unwrap();
    let item = use_current.item;

    let devices = spotify.device().await.expect("Could not get deivces");
    let device = devices
        .iter()
        .find(|device| device.is_active)
        .unwrap()
        .id
        .clone();

    let shuffle = spotify
        .current_playback(None, None::<Vec<_>>)
        .await
        .unwrap()
        .unwrap()
        .shuffle_state;
    let item_id = item.as_ref().unwrap().id().unwrap().id().to_string();
    let track_id = TrackId::from_id(&item_id).unwrap();
    let track_id2 = track_id.clone();
    let liked = spotify
        .current_user_saved_tracks_contains([track_id])
        .await
        .unwrap()[0];
    let track = spotify.track(track_id2, None).await.unwrap();
    let duration = track.duration;
    let time_left = duration - use_current.progress.unwrap();
    let last_update = Instant::now();

    let percentage = (duration.num_milliseconds() - time_left.num_milliseconds()) as f32
        / duration.num_milliseconds() as f32;
    State {
        is_playing,
        progress_ms,
        item,
        device,
        shuffle,
        liked,
        time_left,
        total_time: duration,
        last_update,
        percentage,
        seek_time: -1.0,
        has_connection: true,
    }
}

pub async fn update_state_item(spotify: &AuthCodeSpotify, state: &mut State) {
    let current = spotify.current_playing(None, None::<Vec<_>>).await;

    let Ok(Some(use_current)) = current else {
        state.has_connection = false;
        return;
    };
    state.has_connection = true;

    state.is_playing = use_current.is_playing;
    state.progress_ms = use_current.progress.unwrap();
    state.item = use_current.item;
    get_shuffle_state(spotify, state).await;
    get_liked_state(spotify, state).await;
    update_time(spotify, state).await;
}
