{ pkgs, ... }:

{
  packages = [ pkgs.rust-analyzer pkgs.typescript-language-server ];

  languages.rust = {
    enable = true;
    channel = "stable";
    # We need to add the language server manually so we can add it to the path correctly
    components = [ "rustc" "cargo" "clippy" "rustfmt" ];
  };
  languages.typescript = {
    enable = true;
  };

  env.RUST_ANALYZER = "${pkgs.rust-analyzer}/bin/rust-analyzer";
}
