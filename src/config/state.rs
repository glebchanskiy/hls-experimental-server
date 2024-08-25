use s3::bucket::Bucket;

// use crate::service::track_service::TrackService;

use super::minio::Minio;

pub struct Buckets {
    pub playlists: Bucket,
    pub draft_playlists: Bucket,
}

impl Buckets {
    pub fn new(playlists: Bucket, draft_playlists: Bucket) -> Self {
        Self {
            playlists,
            draft_playlists,
        }
    }
}

pub struct AppState {
    pub minio: Minio,
    // pub track_service: TrackService
}
