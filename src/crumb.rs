use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::collections::btree_map::BTreeMap;

use toml::from_str;
use serde::Deserialize;

use flate2::write::GzEncoder;
use tar::Builder as TarBuilder;
use flate2::Compression;

use crate::style::pkg_name;
use std::process::Command;

#[derive(Deserialize)]
pub struct CrumblePackageInfo {
    name: String,
    description: Option<String>,
    version: f32,
    license: Option<String>,
    homepage: Option<Vec<String>>,
    authors: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct ScriptInfo {
    // basically executes anything like `make`
    install: Option<String>,
    uninstall: Option<String>,
    build: Option<String>,
}

#[derive(Deserialize)]
pub struct CrumbInfo {
    package: CrumblePackageInfo,
    scripts: ScriptInfo,

    dependencies: Option<BTreeMap<String, String>>,
    ignore: Option<BTreeMap<String, bool>>,
}

impl CrumbInfo {
    pub fn from_file<P: AsRef<Path>>(path: P) -> std::io::Result<CrumbInfo> {
        let path = path.as_ref();
        let mut file = File::open(path)?;

        let mut toml_data = String::default();
        file.read_to_string(&mut toml_data)?;

        Ok(CrumbInfo::from_string(toml_data))
    }

    pub fn from_string<S: AsRef<str>>(data: S) -> CrumbInfo {
        let crumb_info: CrumbInfo = from_str(data.as_ref()).unwrap(); // TODO: print error if this fails.

        if crumb_info.package.description.is_none() {
            log::warn!("description is not set in [package]");
        }

        if crumb_info.package.license.is_none() {
            log::warn!("license is not set in [package]");
        }

        if crumb_info.package.license.is_none() {
            log::warn!("homepage[] is not set in [package]");
        }

        if crumb_info.package.license.is_none() {
            log::warn!("authors[] is not set in [package]");
        }


        if crumb_info.scripts.install.is_none() {
            log::warn!("install is not set in [scripts]");
        }

        if crumb_info.scripts.uninstall.is_none() {
            log::warn!("uninstall is not set in [scripts]");
        }

        if crumb_info.scripts.build.is_none() {
            log::warn!("build is not set in [scripts]");
        }

        if crumb_info.dependencies.is_none() {
            log::warn!("No [dependencies] are being used");
        }

        if crumb_info.ignore.is_none() {
            log::warn!("[ignore] is not set");
        }

        crumb_info
    }
}

pub struct Crumb {

}

impl Crumb {
    pub fn bake_package<P: AsRef<Path>>(path: P) {
        let path = path.as_ref();

        log::trace!("Reading {}", pkg_name("crumb.toml"));
        let crumb_info = CrumbInfo::from_file(path.join("crumb.toml"));
        if crumb_info.is_err() {
            let err = crumb_info.err().unwrap();

            log::error!("Failed to read crumb.toml {}", err);
            std::process::exit(1);
        }

        let info = crumb_info.unwrap();

        let mut package_name = String::default();
        package_name += info.package.name.as_str();
        package_name += "@";
        package_name += info.package.version.to_string().as_str();
        package_name += ".crumb";

        if info.scripts.build.is_some() {
            let build_script = info.scripts.build.unwrap();
            log::trace!("Executing {}", pkg_name(&build_script));

            Command::new("sh")
                .arg("-c")
                .arg(format!("cd {}", path.to_str().unwrap()))
                .arg("&&")
                .arg(build_script)
                .output()
                .expect("failed to execute process");
        }

        log::trace!("Opening {}", package_name);
        let tar_gz = File::create(path.join(&package_name)).unwrap(); // TODO: check for error

        let gz = GzEncoder::new(tar_gz, Compression::best());
        let mut tar = TarBuilder::new(gz);

        log::trace!("Attaching {} if possible", pkg_name("source"));
        let appended = tar.append_dir_all("/", path.join("source"));
        if appended.is_err() {
            log::trace!("{}", pkg_name("Failed"));
        }

        let dir = path.read_dir().unwrap(); // TODO: check for error

        let mut to_ignore = BTreeMap::new();
        if info.ignore.is_some() {
            to_ignore = info.ignore.unwrap();
        }

        for file in dir {
            let file = file.unwrap();
            let file_name_os = file.file_name();
            let file_name = file_name_os.to_str().unwrap();

            if file_name == package_name {
                continue;
            }

            log::trace!("Attaching {}", pkg_name(file_name));

            if !to_ignore.contains_key(file_name) {
                tar.append_path_with_name(file.path(), file_name).unwrap();
            }
        }
    }
}

#[test]
fn parse_crumb_info() {
    let toml_data = r#"
        [package]
        name        = "linux-fs"
        description = "Linux Filesystem"
        version     = "1.0"

        [scripts]
        install   = "./install.sh"
        uninstall = "./uninstall.sh"
        build     = "./build.sh"

        [dependencies]
    "#;

    let crumb_info = CrumbInfo::from_string(toml_data);

    assert_eq!("linux-fs".to_string(), crumb_info.package.name);
    assert_eq!(Some("Linux Filesystem".to_string()), crumb_info.package.description);
    assert_eq!(1.0, crumb_info.package.version);

    assert_eq!(Some("./install.sh".to_string()), crumb_info.scripts.install);
    assert_eq!(Some("./uninstall.sh".to_string()), crumb_info.scripts.uninstall);
    assert_eq!(Some("./build.sh".to_string()), crumb_info.scripts.build);
}
