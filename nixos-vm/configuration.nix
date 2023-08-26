{ pkgs, ... }:
{

  system.stateVersion = "23.05";
  boot.initrd.availableKernelModules = [ "virtio_net" "virtio_pci" "virtio_mmio" "virtio_blk" "virtio_scsi" "9p" "9pnet_virtio" ];
  boot.initrd.kernelModules = [ "virtio_balloon" "virtio_console" "virtio_rng" ];

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
