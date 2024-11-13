{
  pkgs,
  lib,
  rustPlatform,
}:
let
  buildInputs = with pkgs; [
    openssl
    libxkbcommon
    fontconfig
    libGL
    wayland
  ];

  car_thang = rustPlatform.buildRustPackage {
    name = "car_thang";
    version = "0.1.0";

    src = [ ./. ];

    cargoLock.lockFile = ./Cargo.lock;

    nativeBuildInputs = with pkgs; [
      pkg-config
    ];

    inherit buildInputs;

    meta = {
      description = "a clone of the Spotify Car Thing UI to bring it back to life!";
      homepage = "https://github.com/BounceU/car_thang";
      license = lib.licenses.mit;
      maintainers = [ ];
    };
  };

  wrapped = pkgs.writeShellScriptBin "car_thang" ''
    export LD_LIBRARY_PATH="${lib.makeLibraryPath buildInputs}"
    # export RUST_BACKTRACE="full"

    exec ${car_thang}/bin/car_thang
  '';
in
pkgs.symlinkJoin {
  name = "car_thang";
  paths = [
    wrapped
    car_thang
  ];
}
