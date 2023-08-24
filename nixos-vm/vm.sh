#!/usr/bin/env bash

left-vm () {
  export SCRIPTPATH=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

  echo building a leftwm virtual machine...
  echo $SCRIPTPATH
  cd "$SCRIPTPATH"
  nixos-rebuild build-vm --flake ../#leftwm
  ./result/bin/run-leftwm-vm
  cd -
}

