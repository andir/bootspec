{ self, pkgs }:
let
  bios = {
    "aarch64-linux" = "${pkgs.OVMF.fd}/FV/QEMU_EFI.fd";
    "x86_64-linux" = "${pkgs.OVMF.fd}/FV/OVMF.fd";
  }.${pkgs.system};
in
(pkgs.nixosTest {
  machine = {
    imports = [ self.nixosModules.bootspec ];
    boot.loader.secureboot = {
      enable = true;
      signingKeyPath = "/path/to/the/signing/key";
      signingCertPath = "/path/to/the/signing/cert";
    };
  };
  testScript = ''
    machine.start();
  '';
}).overrideAttrs(_: {
  QEMU_OPTS = "-bios=${bios}";
})
