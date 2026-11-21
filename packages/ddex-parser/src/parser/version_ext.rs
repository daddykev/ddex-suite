use ddex_core::models::versions::ERNVersion;

pub trait ERNVersionExt {
    fn namespace_uri(&self) -> &str;
}

impl ERNVersionExt for ERNVersion {
    fn namespace_uri(&self) -> &str {
        self.namespace()
    }
}
