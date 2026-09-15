{
  description = "xwww, a Wayland wallpaper daemon";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    ...
  }: let
    inherit (nixpkgs) lib;
    systems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];
    pkgsFor = lib.genAttrs systems (system:
      import nixpkgs {
        localSystem.system = system;
        overlays = [(import rust-overlay)];
      });
    cargoToml = lib.importTOML ./Cargo.toml;
    inherit (cargoToml.workspace.package) rust-version;
  in {
    packages =
      lib.mapAttrs (system: pkgs: {
        xwww = let
          rust = pkgs.rust-bin.stable.${rust-version}.default;

          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust;
            rustc = rust;
          };
        in
          rustPlatform.buildRustPackage {
            pname = "xwww";

            src = pkgs.nix-gitignore.gitignoreSource [] ./.;
            inherit (cargoToml.workspace.package) version;

            cargoLock.lockFile = ./Cargo.lock;

            buildInputs = with pkgs; [
              lz4
              libxkbcommon
              wayland-scanner
              wayland-protocols
            ];

            doCheck = false; # Integration tests do not work in sandbox environment

            nativeBuildInputs = with pkgs; [
              pkg-config
              installShellFiles
              scdoc
            ];

            postInstall = ''
              for f in doc/*.scd; do
                local page="doc/$(basename "$f" .scd)"
                scdoc < "$f" > "$page"
                installManPage "$page"
              done

              installShellCompletion --cmd xwww \
                --bash completions/xwww.bash \
                --fish completions/xwww.fish \
                --zsh completions/_xwww
            '';

            meta = {
              description = "Efficient animated wallpaper daemon for wayland, controlled at runtime";
              license = lib.licenses.gpl3;
              platforms = lib.platforms.linux;
              mainProgram = "xwww";
            };
          };

        default = self.packages.${system}.xwww;
      })
      pkgsFor;

    formatter = lib.mapAttrs (_: pkgs: pkgs.alejandra) pkgsFor;

    devShells =
      lib.mapAttrs (system: pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [self.packages.${system}.xwww];

          packages = [pkgs.rust-bin.stable.${rust-version}.default];
        };
      })
      pkgsFor;

    overlays.default = final: prev: {inherit (self.packages.${prev.system}) xwww;};
  };
}
