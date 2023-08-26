#!/usr/bin/env bash

left-vm () {
  export SCRIPTPATH=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

  if [ "$1" = "clean" ]; then 

    rm -rf "${SCRIPTPATH}/result"
    rm "${SCRIPTPATH}/leftwm.qcow2"
    
  else
  
    echo building a leftwm virtual machine...
    echo $SCRIPTPATH
    cd "$SCRIPTPATH" # nixos-rebuild build-vm dumps result into current directory

    export QEMU_OPTS="
      -vga qxl
      -spice port=5930,disable-ticketing=on
      -device virtio-serial-pci
      -chardev spicevmc,id=spicechannel0,name=vdagent
      -device virtserialport,chardev=spicechannel0,name=com.redhat.spice.0
    "
    nixos-rebuild build-vm --flake ../#leftwm
    ./result/bin/run-leftwm-vm & PID_QEMU="$!"
    sleep 1
    remote-viewer spice://127.0.0.1:5930
    kill $PID_QEMU
    cd -

  fi
}

