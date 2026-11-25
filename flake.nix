{
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
  };

  outputs =
    { nixpkgs, ... }:

    let
      supportedSystems = [
        "x86_64-linux"
      ];

      forEachSupportedSystem =
        function:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          function {
            inherit system;

            pkgs = import nixpkgs {
              inherit system;
            };
          }
        );
    in
    {
      devShells = forEachSupportedSystem (
        { pkgs, ... }:
        let
          rustToolchain = pkgs.buildEnv {
            name = "kryon-rust-toolchain";
            paths = with pkgs; [
              rustc
              cargo
              rustfmt
              clippy
              rust-analyzer
            ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = [
              rustToolchain
            ]
            ++ (with pkgs; [
              cargo-deny
              cargo-edit
              cargo-nextest
              cargo-watch

              jdk25

              clang
              llvmPackages.libclang
              lld
              cmake
              ninja
              pkg-config
              openssl
              zlib
              git
              just
            ]);

            JAVA_HOME = "${pkgs.jdk25}/lib/openjdk";
            JDK_HOME = "${pkgs.jdk25}/lib/openjdk";
            RUST_TOOLCHAIN_ROOT = "${rustToolchain}";
            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
              pkgs.stdenv.cc.cc.lib
              pkgs.openssl
              pkgs.zlib
            ];
          };
        }
      );

      formatter = forEachSupportedSystem (
        { pkgs, ... }:
        pkgs.nixfmt
      );
    };
}
