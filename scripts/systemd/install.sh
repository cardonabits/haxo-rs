#! /bin/bash

SERVICE=haxo

cp ${SERVICE}.service /etc/systemd/system
mkdir -p /usr/share/haxo
cp ../../notemap.json /usr/share/haxo
systemctl enable ${SERVICE}
