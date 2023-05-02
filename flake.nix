{
  description = "block game icbm radar";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils }:
    let
      description = "block game icbm radar";
      src = ./.;
      packages = flake-utils.lib.eachDefaultSystem (system:
        let
          pkgs = nixpkgs.legacyPackages."${system}";
          craneLib = crane.lib.${system};
        in rec {
          packages.loremipsum-server = craneLib.buildPackage {
            pname = "server";
            version = "0.1.0";
            src = src;
            extraCargoFlags = "-p server";
            doCheck = false;

            buildInputs = with pkgs; [ pkg-config ];
            nativeBuildInputs = with pkgs; [ openssl ];
          };

          packages.loremipsum-client = craneLib.buildPackage {
            pname = "client";
            version = "0.1.0";
            src = src;
            extraCargoFlags = "-p client";
            doCheck = false;

            buildInputs = with pkgs; [ pkg-config ];
            nativeBuildInputs = with pkgs; [ openssl ];
          };

          apps.loremipsum-server = flake-utils.lib.mkApp {
            name = "loremipsum-server";
            drv = packages.loremipsum-server;
          };

          apps.loremipsum-client = flake-utils.lib.mkApp {
            name = "loremipsum-client";
            drv = packages.loremipsum-client;
          };

          devShell = pkgs.mkShell {
            inputsFrom =
              [ packages.loremipsum-server packages.loremipsum-client ];
            nativeBuildInputs = with pkgs; [
              rustc
              cargo
              rust-analyzer
              sqlx-cli
            ];
          };
        });
    in packages // {
      nixosModule = { config, lib, pkgs, ... }:
        with lib;
        let
          cfgServer = config.services.loremipsum-server;
          pkgServer = self.packages.${pkgs.system}.loremipsum-server;
          cfgClient = config.services.loremipsum-client;
          pkgClient = self.packages.${pkgs.system}.loremipsum-client;
        in {
          options.services.loremipsum-server = {
            enable = mkEnableOption description;

            port = mkOption {
              type = types.int;
              default = 56552;
            };

            schedules = mkOption {
              type = types.attrsOf types.str;
              default = { };
            };
          };

          options.services.loremipsum-client = {
            enable = mkEnableOption description;

            configPath = mkOption {
              type = types.str;
              default = "/etc/loremipsum-client.toml";
            };
          };

          config = (mkIf cfgServer.enable {
            users = {
              users.loremipsum = {
                isSystemUser = true;
                group = "loremipsum";
              };

              groups.loremipsum = { };
            };

            systemd.services.loremipsum-server = let
              cfgFile = pkgs.writeText "config.toml" ''
                schedules = { ${
                  concatStringsSep ", "
                  (mapAttrsToList (name: schedule: ''${name} = "${schedule}"'')
                    cfgServer.schedules)
                } }

                [database]
                host = "localhost"
                port = 5432
                username = "loremipsum"
                password = "loremipsum"
                database = "loremipsum"

                [api]
                address = "0.0.0.0:${toString cfgServer.port}"
              '';
            in {
              description = "loremipsum server";
              after = [ "network.target" ];
              wantedBy = [ "multi-user.target" ];

              serviceConfig = {
                User = "loremipsum";
                Group = "loremipsum";
                ExecStart = "${pkgServer}/bin/server ${cfgFile}";
              };
            };

            services.postgresql = {
              enable = mkDefault true;
              authentication = ''
                local loremipsum loremipsum trust
                host loremipsum loremipsum localhost trust
              '';
              ensureDatabases = [ "loremipsum" ];

              ensureUsers = [{
                name = "loremipsum";
                ensurePermissions."DATABASE \"loremipsum\"" = "ALL PRIVILEGES";
              }];

              package = lib.mkForce pkgs.postgresql_13;
              extraPlugins = with pkgs.postgresql_13.pkgs; [ timescaledb ];
              settings.shared_preload_libraries = "timescaledb";
            };
          }) // (mkIf cfgClient.enable {
            systemd.services.loremipsum-client = {
              description = "loremipsum client";
              after = [ "network.target" ];
              wantedBy = [ "multi-user.target" ];

              serviceConfig = {
                ExecStart = "${pkgClient}/bin/client ${cfgClient.configPath}";
              };
            };
          });
        };
    };
}
