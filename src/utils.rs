use std::fmt;

pub struct Forge {
    name: &'static str,
    baseurl: &'static str,
}

impl Forge {
    pub const fn new(name: &'static str, baseurl: &'static str) -> Forge {
        Forge { name, baseurl }
    }
}

// https://stackoverflow.com/questions/32710187/how-do-i-get-an-enum-as-a-string
impl std::fmt::Display for Forge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt::Display::fmt(&self.name, f)
    }
}

// https://github.com/nixos/nixpkgs/archive/78ee0abaa454bc057b6e5623b188b9f4b87be24a.tar.gz
pub const GITHUB: Forge = Forge::new("github", "https://github.com/{}/{}/archive/{}");
// https://gitlab.com/ethancedwards/dotfiles/-/archive/9c694310c38d4c1e73e56e10ef0aab1ee2601897.tar.gz
pub const GITLAB: Forge = Forge::new("gitlab", "https://gitlab.com/{}/{}/-/archive/{}");
// https://git.sr.ht/~misterio/nix-colors/archive/81c0629d3a9a77e2a1d0b381a91760e34149a97d.tar.gz
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

    fn tofunix() -> Tarball {
        Tarball {
            forge: (),
            owner: (),
            repo: (),
            archive: (),
        }
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
        )
    }
}
