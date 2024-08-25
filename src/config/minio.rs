use super::state::Buckets;

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use s3::BucketConfiguration;

static MINIO_ENDPOINT: &str = "http://127.0.0.1:9000";
static MINIO_REGION: &str = "";
static MINIO_ACCESS_KEY: &str = "minio1234567890";
static MINIO_SECRET_KEY: &str = "minio1234567890";

pub struct Minio {
    pub buckets: Buckets,
}

impl Minio {
    pub fn new() -> Self {
        let playlists_bucket = Bucket::new_with_path_style(
            "playlists",
            Region::Custom {
                region: MINIO_REGION.to_owned(),
                endpoint: MINIO_ENDPOINT.to_owned(),
            },
            Credentials {
                access_key: Some(MINIO_ACCESS_KEY.to_owned()),
                secret_key: Some(MINIO_SECRET_KEY.to_owned()),
                security_token: None,
                session_token: None,
            },
        )
        .unwrap();

        let draft_playlists_bucket = Bucket::new_with_path_style(
            "draftplaylists",
            Region::Custom {
                region: MINIO_REGION.to_owned(),
                endpoint: MINIO_ENDPOINT.to_owned(),
            },
            Credentials {
                access_key: Some(MINIO_ACCESS_KEY.to_owned()),
                secret_key: Some(MINIO_SECRET_KEY.to_owned()),
                security_token: None,
                session_token: None,
            },
        )
        .unwrap();

        Self {
            buckets: Buckets::new(playlists_bucket, draft_playlists_bucket),
        }
    }

    pub async fn resolve_buckets(&self) {
        self.resolve_bucket(&self.buckets.playlists).await;
        self.resolve_bucket(&self.buckets.draft_playlists).await;
    }

    async fn resolve_bucket(&self, bucket: &Bucket) {
        let (_, code) = bucket.head_object("/").await.unwrap();

        if code == 404 {
            let create_result = Bucket::create_with_path_style(
                bucket.name.as_str(),
                bucket.region.clone(),
                bucket.credentials.clone(),
                BucketConfiguration::default(),
            )
            .await
            .expect(&format!(
                "[error] Error while creating bucket {}",
                bucket.name
            ));

            println!(
                "[info] === Bucket created\n{} - {} - {}",
                bucket.name, create_result.response_code, create_result.response_text
            );
        }
    }
}
