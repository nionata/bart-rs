{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forEachSystem = nixpkgs.lib.genAttrs systems;
    in
    {
      formatter = forEachSystem (system: nixpkgs.legacyPackages.${system}.nixfmt-tree);

      devShells = forEachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          # Web / native dev (Leptos, bart-cli)
          webToolchain = fenix.packages.${system}.combine [
            fenix.packages.${system}.stable.cargo
            fenix.packages.${system}.stable.rustc
            fenix.packages.${system}.stable.rustfmt
            fenix.packages.${system}.stable.clippy
            fenix.packages.${system}.stable.rust-src
            fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.rust-std
          ];

          # ESP32-C3 firmware (no_std, RISC-V bare metal)
          esp32c3Toolchain = fenix.packages.${system}.combine [
            fenix.packages.${system}.stable.cargo
            fenix.packages.${system}.stable.rustc
            fenix.packages.${system}.stable.rustfmt
            fenix.packages.${system}.stable.clippy
            fenix.packages.${system}.stable.rust-src
            fenix.packages.${system}.targets."riscv32imc-unknown-none-elf".stable.rust-std
          ];
        in
        {
          default = pkgs.mkShell {
            packages = [
              webToolchain
              pkgs.trunk
            ];
          };

          esp32c3 = pkgs.mkShell {
            packages = [
              esp32c3Toolchain
              pkgs.espflash
            ];
            # Write build artifacts into the project tree so macOS can reach them.
            # The default Nix cargo setup redirects target/ to /var/cache/cargo/target.
            shellHook = ''
              export CARGO_TARGET_DIR="$(pwd)/target"
            '';
          };
        }
      );
    };
}
