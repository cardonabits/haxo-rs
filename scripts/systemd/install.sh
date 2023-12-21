#! /bin/bash

SERVICE=haxo

cp ${SERVICE}.service /etc/systemd/system
cp blink-zero.service /etc/systemd/system
mkdir -p /usr/share/haxo
cp ../../notemap.json /usr/share/haxo
cp ../../midi/startup/Startup_Haxophone.mid /usr/share/haxo
grep g_midi /etc/modules || echo g_midi >> /etc/modules
systemctl enable ${SERVICE}
systemctl enable blink-zero.service
