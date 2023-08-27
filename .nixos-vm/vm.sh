#!/usr/bin/env bash

left-vm () {
  export SCRIPTPATH=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

  if [ "$1" = "clean" ]; then 

    rm -rf "${SCRIPTPATH}/result"
    rm "${SCRIPTPATH}/leftwm.qcow2"
    
  else
  
    echo building a leftwm virtual machine...
    cd "$SCRIPTPATH" # nixos-rebuild build-vm dumps result into current directory

    nixos-rebuild build-vm --flake ../#leftwm

    export QEMU_OPTS="
      -vga qxl
      -spice unix=on,addr=/tmp/vm_spice.socket,disable-ticketing=on
      -device virtio-serial-pci
      -chardev spicevmc,id=spicechannel0,name=vdagent
      -device virtserialport,chardev=spicechannel0,name=com.redhat.spice.0
    "
    ./result/bin/run-leftwm-vm & PID_QEMU="$!"
    sleep 1
    remote-viewer spice+unix:///tmp/vm_spice.socket
    kill $PID_QEMU
    cd -

    unset QEMU_OPTS
  fi

  unset SCRIPTPATH
}

