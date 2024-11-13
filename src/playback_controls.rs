use std::time::{Duration, Instant};

use chrono::TimeDelta;
use rspotify::{
    model::{Id, PlayableItem, TrackId},
    prelude::{BaseClient, OAuthClient},
    AuthCodeSpotify, ClientError,
};

pub struct State {
    pub is_playing: bool,
    pub progress_ms: TimeDelta,
    pub item: Option<PlayableItem>,
    pub device: Option<String>,
    pub shuffle: bool,
    pub liked: bool,
    pub time_left: TimeDelta,
    pub total_time: TimeDelta,
    pub last_update: Instant,
    pub percentage: f32,
    pub seek_time: f32,
    pub has_connection: bool,
}

pub async fn change_playback_state(
    spotify: &AuthCodeSpotify,
    state: &mut State,
) -> Result<(), ClientError> {
    let device = state.device.clone();

    if state.is_playing {
        state.is_playing = false;
        spotify.pause_playback(device.as_deref()).await
    } else {
        state.is_playing = true;
        state.last_update = Instant::now();
        spotify.resume_playback(device.as_deref(), None).await
    }
}

pub async fn seek_in_track(spotify: &AuthCodeSpotify, state: &mut State) {
    let new_time = (state.seek_time / 800.0 * state.total_time.num_milliseconds() as f32) as u64;
    let time = TimeDelta::from_std(Duration::from_millis(new_time)).unwrap();
    let device = state.device.clone();
    let _ = spotify.seek_track(time, device.as_deref()).await;
    state.last_update = Instant::now();
    state.time_left = state.total_time - time;
    state.percentage = new_time as f32;
}

pub async fn update_time(spotify: &AuthCodeSpotify, state: &mut State) {
    let current = spotify
        .current_playing(None, None::<Vec<_>>)
        .await
        .unwrap()
        .unwrap();

    let binding = state.item.clone().unwrap();
    let binding = binding.id().unwrap();
    let track_id = TrackId::from_id(binding.id()).unwrap();
    let track = spotify.track(track_id, None).await.unwrap();

    let duration = track.duration;
    let time_left = duration - current.progress.unwrap();
    let last_update = Instant::now();

    state.time_left = time_left;
    state.total_time = duration;
    state.last_update = last_update;
    let percentage = (duration.num_milliseconds() - time_left.num_milliseconds()) as f32
        / duration.num_milliseconds() as f32;
    state.percentage = percentage;
}

pub async fn skip_forward(spotify: &AuthCodeSpotify, state: &mut State) -> Result<(), ClientError> {
    let device = state.device.clone();
    spotify.next_track(device.as_deref()).await
}

pub async fn skip_backward(
    spotify: &AuthCodeSpotify,
    state: &mut State,
) -> Result<(), ClientError> {
    let device = state.device.clone();
    spotify.previous_track(device.as_deref()).await
}

pub async fn shuffle(spotify: &AuthCodeSpotify, state: &mut State) -> Result<(), ClientError> {
    let device = state.device.clone();
    let shuffle_state = spotify.shuffle(!state.shuffle, device.as_deref()).await;
    state.shuffle = !state.shuffle;
    shuffle_state
}

pub async fn get_name_of_current_song(
    spotify: &AuthCodeSpotify,
    state: &mut State,
) -> Result<(String, String, String), ClientError> {
    let binding = state.item.clone().unwrap();
    let binding = binding.id().unwrap();
    let track_id = TrackId::from_id(binding.id()).unwrap();
    let track = spotify.track(track_id, None).await.unwrap();
    Ok((
        track.name,
        track
            .artists
            .into_iter()
            .map(|x| x.name)
            .collect::<Vec<_>>()
            .join(", "),
        track.album.name,
    ))
}

pub async fn get_shuffle_state(spotify: &AuthCodeSpotify, state: &mut State) -> bool {
    let _ = state;
    spotify
        .current_playback(None, None::<Vec<_>>)
        .await
        .unwrap()
        .unwrap()
        .shuffle_state
}

pub async fn get_liked_state(spotify: &AuthCodeSpotify, state: &mut State) -> bool {
    let item_id = state.item.as_ref().unwrap().id().unwrap().id().to_string();
    let track_id = TrackId::from_id(&item_id).unwrap();
    state.liked = spotify
        .current_user_saved_tracks_contains([track_id])
        .await
        .unwrap()[0];
    state.liked
}

pub async fn like(spotify: &AuthCodeSpotify, state: &mut State) -> Result<(), ClientError> {
    let item_id = state.item.as_ref().unwrap().id().unwrap().id().to_string();
    let track_id = TrackId::from_id(&item_id).unwrap();
    get_liked_state(spotify, state).await;
    if state.liked {
        state.liked = false;
        spotify.current_user_saved_tracks_delete([track_id]).await
    } else {
        state.liked = true;
        spotify.current_user_saved_tracks_add([track_id]).await
    }
}

pub async fn get_image_url_of_current_song(
    spotify: &AuthCodeSpotify,
    state: &mut State,
) -> Result<String, ClientError> {
    let binding = state.item.clone().unwrap();
    let binding = binding.id().unwrap();
    let track_id = TrackId::from_id(binding.id()).unwrap();
    let track = spotify.track(track_id, None).await.unwrap();
    let img = track.album.images[0].url.clone();
    Ok(img)
}

pub async fn get_current_queue(spotify: &AuthCodeSpotify) -> Vec<String> {
    spotify
        .current_user_queue()
        .await
        .unwrap()
        .queue
        .iter()
        .map(|x| x.id().unwrap().id().to_owned())
        .collect::<Vec<_>>()
}

#[allow(dead_code)]
pub async fn get_current_history(spotify: &AuthCodeSpotify) -> Vec<String> {
    spotify
        .current_user_recently_played(Some(50), None)
        .await
        .unwrap()
        .items
        .iter()
        .map(|item| item.track.id.clone().unwrap().id().to_owned())
        .collect::<Vec<_>>()
}

pub async fn get_buffer_and_location(spotify: &AuthCodeSpotify) -> (Vec<String>, usize) {
    let queue = get_current_queue(spotify).await;

    (queue, 0)
}

pub async fn weak_update_state_item(
    spotify: &AuthCodeSpotify,
    state: &mut State,
    queue: &[String],
    loc_in_queue: &mut usize,
) {
    let track_id = TrackId::from_id(&queue[*loc_in_queue]).unwrap();
    let b = spotify.track(track_id, None).await.unwrap();
    state.item = Some(PlayableItem::Track(b));
    get_shuffle_state(spotify, state).await;
    get_liked_state(spotify, state).await;
    update_time(spotify, state).await;
}

pub async fn guess_current_progress(spotify: &AuthCodeSpotify, state: &mut State) {
    let _ = spotify;
    let now = Instant::now();
    let time_passed = TimeDelta::from_std(now - state.last_update).unwrap();

    if !state.is_playing {
        state.last_update = now;
        let temp = (state.total_time.num_milliseconds() as f64
            - state.percentage as f64 * state.total_time.num_milliseconds() as f64)
            as i64;
        let temp2 = TimeDelta::milliseconds(temp);
        state.time_left = temp2;
    } else {
        state.percentage = (state.total_time.num_milliseconds()
            - (state.time_left.num_milliseconds() - time_passed.num_milliseconds()))
            as f32
            / state.total_time.num_milliseconds() as f32;
    }
}
