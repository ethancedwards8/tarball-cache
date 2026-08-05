{
  pkgs ? import <nixpkgs> { }
}:

rec {
  github = pkgs.fetchurl {
    url = "https://github.com/nixos/nixpkgs/archive/78ee0abaa454bc057b6e5623b188b9f4b87be24a.tar.gz";
    hash = "sha256-PNlBy3B4RNLEacMKCCtBAQC0moU51+UorUw3xg5AsnE=";
  };

  cache = pkgs.fetchurl {
    url = "http://localhost:3000/github/nixos/nixpkgs/78ee0abaa454bc057b6e5623b188b9f4b87be24a.tar.gz";
    inherit (github) hash;
  };
}
