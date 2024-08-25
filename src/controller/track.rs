use rand::{distributions::Alphanumeric, Rng};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::Command,
};

use actix_easy_multipart::{tempfile::Tempfile, MultipartForm};
use actix_web::{get, post, web, HttpResponse, Responder};

use crate::{config::state::AppState, HLS_INDEX_FILE_MEDIA_TYPE, HLS_PART_FILE_MEDIA_TYPE};

#[derive(MultipartForm)]
struct Upload {
    #[multipart(rename = "file")]
    file: Vec<Tempfile>,
}

#[get("/tracks/{id}/index.mp3u8")]
async fn get_playlist(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    println!("KNOCK KNOCK index");
    let minio_id: String = format!("{}/index.m3u8", path.into_inner());

    let (data, _code) = state
        .minio
        .buckets
        .draft_playlists
        .get_object(minio_id)
        .await
        .unwrap();

    HttpResponse::Ok()
        .content_type(HLS_INDEX_FILE_MEDIA_TYPE)
        .body(data)
}

#[get("/tracks/{id}/{part_num}")]
async fn get_part(state: web::Data<AppState>, path: web::Path<(String, String)>) -> impl Responder {
    println!("KNOCK KNOCK parts");
    let (id, part_num) = path.into_inner();

    let part_name = &part_num[part_num.len() - 10..part_num.len()];

    println!("part: {}", part_name);
    let minio_id = format!("{}/{}", id, part_name);

    let (data, _code) = state
        .minio
        .buckets
        .draft_playlists
        .get_object(minio_id)
        .await
        .unwrap();

    HttpResponse::Ok()
        .content_type(HLS_PART_FILE_MEDIA_TYPE)
        .body(data)
}

#[post("/upload-track")]
async fn upload(state: web::Data<AppState>, mut form: MultipartForm<Upload>) -> impl Responder {
    let mut buffer = Vec::new();
    let _ = form.file[0].file.read_to_end(&mut buffer);
    let data = &buffer[..];

    let filename = form.file[0]
        .file_name
        .get_or_insert("NONE".to_string())
        .replace("'", "")
        .replace(" ", "_")
        .replace("-", "_")
        .to_lowercase()
        .to_string();

    let temp_folder_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let temp_folder_path = ["workdir", &temp_folder_name].join("/").to_string();

    let file_path = &format!("{}/{}", &temp_folder_path, &filename);
    let _ = fs::create_dir_all(Path::new(&temp_folder_path));

    let mut file = File::create(Path::new(&file_path)).unwrap();
    let _ = file.write_all(data);

    let command = format!("
        ffmpeg -i {:?} -map 0:a -vn -ac 2 -acodec aac -f segment -segment_format mpegts -segment_time 10 \
        -segment_list {:?}.index.m3u8 {:?}.part%03d.ts -f ffmetadata {:?}.metadata.txt", 
        Path::new(&file_path),
        Path::new(&file_path),
        Path::new(&file_path),
        Path::new(&file_path),
    );

    let _cmd = Command::new("sh")
        .arg("-c")
        .arg(command)
        // .stdout(Stdio::piped())
        // .spawn()
        .output()
        .unwrap();

    let paths = fs::read_dir(Path::new(&temp_folder_path)).unwrap();

    for path in paths {
        let path = path.unwrap();
        let directory_file_name = &path.file_name().into_string().unwrap();
        let directory_file_path = &path.path();

        let is_index_file = directory_file_name.ends_with(".index.m3u8");
        let is_part_file = directory_file_name.ends_with(".ts");

        if is_index_file || is_part_file {
            let mut file = File::open(&directory_file_path).unwrap();
            let mut buffer = Vec::new();
            let _ = file.read_to_end(&mut buffer);
            let file_data = &buffer[..];

            if is_index_file {
                let _ = state
                    .minio
                    .buckets
                    .draft_playlists
                    .put_object_with_content_type(
                        &format!("{}/index.m3u8", &filename),
                        file_data,
                        HLS_INDEX_FILE_MEDIA_TYPE,
                    )
                    .await;
            } else {
                let _ = state
                    .minio
                    .buckets
                    .draft_playlists
                    .put_object_with_content_type(
                        &format!(
                            "{}/{}",
                            &filename,
                            &directory_file_name
                                [directory_file_name.len() - 10..directory_file_name.len()]
                        ),
                        file_data,
                        HLS_PART_FILE_MEDIA_TYPE,
                    )
                    .await;
            }
        }
    }

    format!("Track ID: {}", filename)
}
