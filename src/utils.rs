use crate::Tarball;

#[inline]
pub fn create_cache_key(tarball: &Tarball) -> String {
    let Tarball {
        forge,
        owner,
        repo,
        archive,
    } = tarball;
    format!("{forge}-{owner}-{repo}-{archive}")
}

#[inline]
pub fn get_bucket_path(tarball: &Tarball) -> String {
    let Tarball {
        forge,
        owner,
        repo,
        archive,
    } = tarball;
    format!("{forge}/{owner}/{repo}/{archive}")
}

#[inline]
pub fn github_url(tarball: &Tarball) -> String {
    #[allow(unused_variables)]
    let Tarball {
        forge,
        owner,
        repo,
        archive,
    } = tarball;
    format!("https://github.com/{owner}/{repo}/archive/{archive}")
}
