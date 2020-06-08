// a bread database is super simple, basically
// $NAME@$VERSION $BYTE_SIZE $FILE_SHA512
// it is required to be a .gz
// a file name would be (database_name).db.gz
// the database would be extracted on the disk at /var/bread/database/{database_name}.db

use std::fs;
use std::path::Path;
use std::time::Instant;

use sha2::{Sha512, Digest};

use log::{trace, info, error};

use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;

use indicatif::{ProgressBar, MultiProgress, TickTimeLimit, ProgressDrawTarget};

use crate::style::{pkg_name, install_pg_style};
use std::io::{Read, Write};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct DatabaseEntry {
    pub name: String,
    pub version: String,

    pub size: u64,
    pub checksum: String, 
}

#[derive(Debug, Clone)]
pub struct Database {
    pub name: String,

    pub entries: Vec<DatabaseEntry>
}

impl DatabaseEntry {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> DatabaseEntry {
        let path = path.as_ref();
        let mut file = fs::File::open(path).unwrap(); // TODO: check for permissions
        let meta = file.metadata().unwrap(); // TODO: check for permissions

        let mut sha512 = Sha512::default();
        std::io::copy(&mut file, &mut sha512).unwrap();
        let hash = hex::encode(sha512.result());

        let file_name = path.file_name().unwrap().to_str().unwrap();
        let file_name_s: Vec<&str> = file_name.split("@").collect();

        let name = file_name_s[0]; // TODO: log an error if file name is in the wrong format
        let version = file_name_s[1].replace(".crumb", "");

        DatabaseEntry {
            name: name.to_string(),
            version: version.to_string(),

            size: meta.len(),
            checksum: hash
        }
    }

    fn new() -> DatabaseEntry {
        DatabaseEntry {
            name: String::default(),
            version: String::default(),
            checksum: String::default(),
            size: 0
        }
    }
}

impl Database {
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref().join(self.name.to_owned() + ".db.gz");
        let file = fs::File::create(&path).unwrap();
        
        let mut end_db_str = String::default();
        end_db_str.push_str("VERSION_1_0\n");
        for entry in &self.entries {
            end_db_str.push_str(format!("{}-{} {} {}\n", entry.name, entry.version, entry.size, entry.checksum).as_str());
        }

        trace!("Saving to {}", pkg_name(path.to_str().unwrap()));

        let mut enc = GzEncoder::new(file, Compression::new(9));
        enc.write_all(end_db_str.as_bytes()).unwrap(); // TODO: check for error
        enc.finish().unwrap(); // TODO: check for error
    }

    pub async fn from_mirror<S: AsRef<str>, SI: AsRef<str>>(uri: S, name: SI) -> Database {
        let name = name.as_ref();
        let uri = uri.as_ref();

        let now = Instant::now();
        
        info!("Fetching {}", pkg_name(name));

        let mut database_url = uri.to_string();
        database_url.push_str("/");
        database_url.push_str(name);
        database_url.push_str(".db.gz");

        let gz = crate::net::download_file(database_url).await.unwrap(); // TODO: check for error
        // TODO: verify SHA512

        info!("Extracting {}", pkg_name(name));

        let mut file = GzDecoder::new(gz);
        let mut buf = String::default();
        file.read_to_string(&mut buf).unwrap(); // TODO: check for errors

        info!("Done, took {}ms", now.elapsed().as_millis());

        Database::from_string(name, buf)
    }

    pub fn from_string<S: AsRef<str>, SI: AsRef<str>>(name: S, data: SI) -> Database {
        let name = name.as_ref();
        let data = data.as_ref();

        let database_rows = data.lines();

        let mut db_major = 0;
        let mut db_minor = 0;

        let mut database_entries = vec![];

        for row in database_rows {
            if row.starts_with("VERSION_") {
                let v = row.replace("VERSION_", "");
                let v_split: Vec<&str> = v.split("_").collect();

                db_major = v_split[0].parse().unwrap(); // TODO: instead of crashing, show the error cause.
                db_minor = v_split[1].parse().unwrap();

                continue;
            }

            let mut row_split = row.split_whitespace();

            let mut entry = DatabaseEntry::new();

            if db_major >= 1 && db_minor >= 0 {
                let name_version: Vec<&str> = row_split.next().unwrap().splitn(2, "@").collect();

                entry.name = name_version[0].to_string();
                entry.size = row_split.next().unwrap().parse().unwrap(); // TODO: instead of crashing, show the error cause.
                entry.checksum = row_split.next().unwrap().to_string();
                entry.version = name_version[1].to_string();
            }

            database_entries.push(entry);
        }

        Database {
            name: name.to_string(),
            entries: database_entries
        }
    }

    pub async fn from_folder<P: AsRef<Path>>(path: P, name: &str, architecture: &str) -> Database {
        let path = path.as_ref();
        let crumbs_path = path.join("crumbs");

        let mut database_entries = vec![];

        if !crumbs_path.exists() {
            error!("{} directory doesn't exists! it's required for building a database.", pkg_name("crumbs"));
            std::process::exit(1);
        } else {
            let architecture_path = crumbs_path.join(architecture); // TODO: check if exists
            let crumbs = architecture_path.read_dir().unwrap(); // TODO: check for read permissions

            let mut i = 0; // there is no length
            for _ in crumbs {
                i += 1;
            }

            let mpb = MultiProgress::new();

            let pb = mpb.add(ProgressBar::new(i));
            pb.set_style(install_pg_style());

            for entry in architecture_path.read_dir().unwrap() { // TODO: check for read permissions
                let entry = entry.unwrap();

                let entry = DatabaseEntry::from_file(entry.path()).await;

                mpb.clear().unwrap();
                mpb.set_draw_target(ProgressDrawTarget::hidden());
                info!("Entry: {}@{}", pkg_name(entry.name.as_str()), entry.version);
                mpb.set_draw_target(ProgressDrawTarget::stdout());

                pb.inc(1);

                mpb.tick_and_clear(TickTimeLimit::Indefinite).unwrap();

                database_entries.push(entry);
            }

            mpb.clear().unwrap();
        }

        Database {
            name: name.to_string(),
            entries: database_entries
        }
    }

    // Query through all databases in path
    // folder structure must look as followed:
    //
    //  /x86_64/{db_name}.gz
    //  /i686/{db_name}.gz
    //
    pub fn query<P: AsRef<Path>, S: AsRef<str>, SI: AsRef<str>>(path: P, pkg_name: S, architecture: SI) {
        let dir = path.as_ref().join(architecture.as_ref()).as_path();
    }

    pub fn query_s<S: AsRef<str>, SI: AsRef<str>>(&self, pkg_name: S, architecture: SI) -> Option<&DatabaseEntry> {
        for entry in &self.entries {
            if entry.name == pkg_name.as_ref() {
                return Some(entry);
            }
        }

        None
    }
}
