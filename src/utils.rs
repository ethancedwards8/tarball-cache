pub struct Forge {
    name: &'static str,
    baseurl: &'static str,
}

impl Forge {
    pub const fn new(name: &'static str, baseurl: &'static str) -> Forge {
        Forge { name, baseurl }
    }
}

pub const GITHUB: Forge = Forge::new("github", "https://github.com/{}/{}/archive/{}");
pub const GITLAB: Forge = Forge::new("gitlab", "https://gitlab.com/{}/{}/-/archive/{}");
pub const CODEBERG: Forge = Forge::new("codeberg", "https://codeberg.org/{}/{}/archive/{}");
pub const SOURCEHUT: Forge = Forge::new("sourcehut", "https://git.sr.ht/~{}/{}/archive/{}");

pub struct Tarball {
    forge: Forge,
    owner: String,
    repo: String,
    archive: String,
}

impl Tarball {
    pub fn new(forge: Forge, owner: String, repo: String, archive: String) -> Tarball {
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
            self.forge.name, self.owner, self.repo, self.archive
        )
    }

    pub fn get_path(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.forge.name, self.owner, self.repo, self.archive
        )
    }

    pub fn get_url(&self) -> String {
        // this feels cursed and messy but idk apparenlty `format!` only accepts
        // compile time strings
        self.forge
            .baseurl
            .replacen("{}", &self.owner, 1)
            .replacen("{}", &self.repo, 1)
            .replacen("{}", &self.archive, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nixpkgs() -> Tarball {
        Tarball::new(
            GITHUB,
            "nixos".to_string(),
            "nixpkgs".to_string(),
            "78ee0abaa454bc057b6e5623b188b9f4b87be24a.tar.gz".to_string(),
        )
    }

    fn gitlab() -> Tarball {
        Tarball::new(
            GITLAB,
            "ethancedwards".to_string(),
            "dotfiles".to_string(),
            "9c694310c38d4c1e73e56e10ef0aab1ee2601897.tar.gz".to_string(),
        )
    }

    fn codeberg() -> Tarball {
        Tarball::new(
            CODEBERG,
            "poz".to_string(),
            "niri-nix".to_string(),
            "da8a388cfc14d55f19992c27e6870d836948bc19.tar.gz".to_string(),
        )
    }

    fn sourcehut() -> Tarball {
        Tarball::new(
            SOURCEHUT,
            "misterio".to_string(),
            "nix-colors".to_string(),
            "81c0629d3a9a77e2a1d0b381a91760e34149a97d.tar.gz".to_string(),
        )
    }

    #[test]
    fn check_key() {
        assert_eq!(
            nixpkgs().get_key(),
            "github-nixos-nixpkgs-78ee0abaa454bc057b6e5623b188b9f4b87be24a.tar.gz"
        )
    }

    #[test]
    fn check_path() {
        assert_eq!(
            nixpkgs().get_path(),
            "github/nixos/nixpkgs/78ee0abaa454bc057b6e5623b188b9f4b87be24a.tar.gz"
        )
    }

    #[test]
    fn check_url() {
        assert_eq!(
            nixpkgs().get_url(),
            "https://github.com/nixos/nixpkgs/archive/78ee0abaa454bc057b6e5623b188b9f4b87be24a.tar.gz"
        );

        assert_eq!(
            sourcehut().get_url(),
            "https://git.sr.ht/~misterio/nix-colors/archive/81c0629d3a9a77e2a1d0b381a91760e34149a97d.tar.gz"
        );

        assert_eq!(
            gitlab().get_url(),
            "https://gitlab.com/ethancedwards/dotfiles/-/archive/9c694310c38d4c1e73e56e10ef0aab1ee2601897.tar.gz"
        );

        assert_eq!(
            codeberg().get_url(),
            "https://codeberg.org/poz/niri-nix/archive/da8a388cfc14d55f19992c27e6870d836948bc19.tar.gz"
        )
    }
}
