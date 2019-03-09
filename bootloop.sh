#!/bin/bash

SCRIPTPATH="$( cd "$(dirname "$0")" ; pwd -P )"

while true
do
  rm ./leftwm.logs.old
  mv ./leftwm.logs ./leftwm.logs.old
  $SCRIPTPATH/target/debug/leftwm &> ./leftwm.logs
  echo "loop" >> /home/lex/loop.log  
  sleep 1
done

