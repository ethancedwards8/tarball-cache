pub struct Tarball {
    forge: String, // should be an enum but will handle later
    owner: String,
    repo: String,
    archive: String,
}

impl Tarball {
    pub fn new(forge: String, owner: String, repo: String, archive: String) -> Tarball {
        Tarball {
            forge,
            owner,
            repo,
            archive,
        }
    }

    pub fn get_key(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.forge, self.owner, self.repo, self.archive
        )
    }

    pub fn get_path(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.forge, self.owner, self.repo, self.archive
        )
    }

    pub fn get_url(&self) -> String {
        format!(
            "https://github.com/{}/{}/archive/{}",
            self.owner, self.repo, self.archive
        )
    }
}
