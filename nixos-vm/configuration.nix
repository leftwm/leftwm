{ pkgs, ... }:
{

  system.stateVersion = "23.05";

  # this willl get overriden
  # just to silence CI
  boot.loader.grub.device = "/dev/vda";
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };

  users.users.leftwm = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    initialPassword = "";
    home = "/home/leftwm";
  };

  networking = {
    hostName = "leftwm";
    networkmanager.enable = true;
  };

  environment.systemPackages = with pkgs; [
    alacritty
  ];

  programs.git.enable = true;
  services = {
    openssh.enable = true;
    spice-vdagentd.enable = true;
    qemuGuest.enable = true;
  
    xserver = {
      videoDrivers = [ "qxl" ];
      enable = true;

      desktopManager.xterm.enable = false;
      displayManager.autoLogin.enable = true;
      displayManager.autoLogin.user = "leftwm";
      windowManager.leftwm.enable = true;
    };  
  };

}
