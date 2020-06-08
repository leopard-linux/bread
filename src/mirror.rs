use hyper::Uri;
use crate::database::Database;

pub struct Mirror {
    pub uri: Uri,
    pub db: Database
}