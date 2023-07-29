{
    description = "a random yew project";

    inputs = {
        rust-overlay.url = github:oxalica/rust-overlay;
        utils.url = github:numtide/flake-utils;
        wasm-tooling.url = github:rambip/wasm-tooling;
    };

    outputs = { self, rust-overlay, nixpkgs, wasm-tooling, utils }: 
        with utils.lib;
        eachSystem [system.x86_64-linux system.x86_64-darwin] (system:
            let overlays = [rust-overlay.overlays.default];
                pkgs = import nixpkgs {inherit system overlays;};
                rust-tools = pkgs.callPackage wasm-tooling.lib."${system}".rust {
                    cargo-toml = ./Cargo.toml;
                    rust-toolchain = pkgs.rust-bin.nightly.latest.minimal;
                };
                build = example_name: rust-tools.buildWithTrunk {
                     src=./.;
                     fixRelativeUrl = true;
                     relativeHtmlTarget = "examples/${example_name}/index.html";
                };
                examples = builtins.readDir ./examples;
                built_examples = builtins.mapAttrs (name: value: build name) examples;
                generate_copy_command = name : ''cp -r ${build name} $out/${name}'';
                copy_commands = builtins.map generate_copy_command (
                   builtins.attrNames (builtins.readDir ./examples)
                );

            in
            {
                packages = built_examples // {
                    default = 
                        nixpkgs.legacyPackages."${system}".stdenv.mkDerivation {
                         name = "markdown examples";
                         src = "/dev/null";
                         phases = [ "installPhase" ];
                         installPhase = ''
                         mkdir $out
                         ${builtins.concatStringsSep "\n" copy_commands}
                         '';
                     };
                };

                devShell = pkgs.mkShell {
                    buildInputs = [pkgs.openssl pkgs.pkg-config] ++ rust-tools.allTools;
                };
            }
    );
}
