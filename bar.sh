#!/bin/bash

while true; do 
  tail ~/leftwm.logs | ag "TAGS" | awk '{$1= ""; print $0}'
  sleep 0.1
done
