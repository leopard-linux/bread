use std::fs::File;
use std::io::Write;
use std::path::Path;

use log::{info, trace};

use hyper::{Client, Response, Body};
use hyper::body::{HttpBody, Buf};
use hyper_tls::HttpsConnector;

use crate::style::{pkg_name, download_pg_style};
use crate::constants::PATH_CACHE;
use indicatif::{ProgressBar, MultiProgress, TickTimeLimit, ProgressDrawTarget};

async fn save_to_file(file_name: &str, pb: &ProgressBar, res: &mut Response<Body>, mpb: Option<&MultiProgress>) -> File {
    let mut file = File::create(PATH_CACHE.to_string() + "/" + file_name).unwrap(); // TODO: print an error instead of crashing

    while let Some(next) = res.data().await {
        let chunk = next.unwrap();
        file.write(chunk.bytes()).unwrap();
        pb.inc(chunk.len() as u64);

        if mpb.is_some() {
            mpb.unwrap().tick_and_clear(TickTimeLimit::Indefinite).unwrap();
        }
    }

    file.flush().unwrap();

    if mpb.is_some() && log::max_level() == log::Level::Trace {
        let mpb = mpb.unwrap();
        mpb.clear().unwrap();
        mpb.set_draw_target(ProgressDrawTarget::hidden());
        trace!("Saving to {}", pkg_name((PATH_CACHE.to_string() + "/" + file_name).as_str()));
        mpb.set_draw_target(ProgressDrawTarget::stdout());
    }

    File::open(PATH_CACHE.to_string() + "/" + file_name).unwrap()
}

pub async fn download_file(uri: String) -> Option<File> {
    trace!("Downloading {}", pkg_name(uri.as_str()));

    //let mut written = 0;
    let url = uri.parse::<hyper::Uri>().unwrap();

    let tmp_path = url.path().to_string();
    let url_path = Path::new(tmp_path.as_str());
    let file_name = url_path.file_name().unwrap().to_str().unwrap();

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let mut res = client.get(url).await.unwrap();
    let expected_length = res.headers().get("Content-Length").unwrap().to_str().unwrap().parse().unwrap();

    let pb = ProgressBar::new(expected_length);
    pb.set_style(download_pg_style());
    pb.set_prefix(&format!("[{}]", file_name));

    let file = save_to_file(file_name, &pb, &mut res, None).await;

    pb.finish_and_clear();
    trace!("Finished downloading {}", pkg_name(file_name));

    Some(file)
}

pub struct MultiDownloadResult {
    pub file: File,
    pub name: String
}

#[allow(dead_code)]
pub async fn download_files(uris: Vec<String>) -> Vec<Option<MultiDownloadResult>> {
    let mpb = MultiProgress::new();

    let mut futures = vec![];
    for url in uris {
        let future = async {
            let moved_url = url;

            mpb.clear().unwrap();
            mpb.set_draw_target(ProgressDrawTarget::hidden());
            trace!("Downloading {}", pkg_name(moved_url.as_str()));
            mpb.set_draw_target(ProgressDrawTarget::stdout());

            let uri = moved_url.parse::<hyper::Uri>().unwrap(); // TODO: check for error

            let tmp_path = uri.path().to_string();
            let url_path = Path::new(tmp_path.as_str());
            let file_name = url_path.file_name().unwrap().to_str().unwrap(); // TODO: check for error

            let https = HttpsConnector::new();
            let client = Client::builder().build::<_, hyper::Body>(https);
            let mut res = client.get(uri).await.unwrap(); // TODO: check for error
            let expected_length: u64 = res.headers().get("Content-Length").unwrap().to_str().unwrap().parse().unwrap(); // TODO: check for error

            let pb = mpb.add(ProgressBar::new(expected_length));
            pb.set_style(download_pg_style());
            pb.set_prefix(&format!("[{}]", file_name));

            let file = save_to_file(file_name, &pb, &mut res, Some(&mpb)).await;

            mpb.clear().unwrap();
            mpb.set_draw_target(ProgressDrawTarget::hidden());
            info!("Finished downloading {}", pkg_name(file_name));
            mpb.set_draw_target(ProgressDrawTarget::stdout());

            mpb.tick_and_clear(TickTimeLimit::Indefinite).unwrap();

            Some(
                MultiDownloadResult {
                    name: file_name.to_string(),
                    file
                }
            )
        };

        futures.push(future);
    }

    let results = futures::future::join_all(futures).await; // join the remaining futures

    mpb.join_and_clear().unwrap(); // lets join them.

    info!("Finished downloading...");

    results
}
