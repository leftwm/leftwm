FROM debian:latest

RUN apt-get update && apt-get install xserver-xorg xorg -y
