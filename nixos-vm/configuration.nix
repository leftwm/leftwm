{ config, pkgs, ... }:
{
  system.stateVersion = "23.05";
  
  users.users.leftwm = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    initialPassword = "";
    home = "/home/leftwm";
  };

  boot.loader.grub.device = "/dev/vda";
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext3";
  };
    
  networking = {
    hostName = "leftwm";
    networkmanager.enable = true;
  };

  environment.systemPackages = with pkgs; [
    alacritty
  ];

  programs.git.enable = true;
  services.openssh.enable = true;
  services.qemuGuest.enable = true;

  services.xserver = {
    enable = true;

    desktopManager.xterm.enable = false;
    displayManager.autoLogin.enable = true;
    displayManager.autoLogin.user = "leftwm";
    windowManager.leftwm.enable = true;
  };  
}
