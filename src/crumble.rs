struct CrumblePackageInfo {
    name: String,
    description: String,
    version: String,
    homepage: Vec<String>,
    authors: Vec<String>,
}

struct ScriptInfo {
    // basically executes anything like `make`
    install: String,
    uninstall: String,
    build: String,
}

struct CrumbleInfo {
    package: CrumblePackageInfo,
    scripts: ScriptInfo,

    dependencies: Vec<String>,
}

struct Crumble {

}

impl Crumble {

}
