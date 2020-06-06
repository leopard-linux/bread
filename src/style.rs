use console::style;
use indicatif::{ProgressStyle};

pub fn pkg_name<S: AsRef<str>>(name: S) -> String {
    format!("{}", style(name.as_ref()).red())
}

pub fn download_pg_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("{prefix:.red} [{wide_bar:>0.cyan}] {bytes:>10.green}/{total_bytes:.green} ({eta:>3.yellow}) {percent:>3}%")
        .progress_chars("▉▊▋▌▍▎▏ ")
}

pub fn install_pg_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("{prefix:.red} [{wide_bar:0.yellow}] ({eta:>3.yellow}) {percent:>3}%")
        .progress_chars("▉▊▋▌▍▎▏ ")
}

pub fn uninstall_pg_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("{prefix:.red} [{wide_bar:0.red}] ({eta:>3.yellow}) {percent:>3}%")
        .progress_chars("▉▊▋▌▍▎▏ ")
}
