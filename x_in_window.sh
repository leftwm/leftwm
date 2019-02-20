#!/bin/bash
set -e

#Xephyr -br -ac -noreset -screen 1024x768 :2
Xephyr -br -ac -noreset -screen 1024x768 -screen 1024x768 :2
#Xephyr -br -ac -noreset -screen 800x600 :2
