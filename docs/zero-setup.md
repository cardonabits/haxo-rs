## Direct connection via Ethernet

This method works on all RPIs with an Ethernet or a usb hub with Ethernet port. 

1. Attach RPI to development host via direct ethernet link.
2. On development host, find out IPv6 link local address of RPI by pinging link
   local multicast address (`ff02::1`) on the local ethernet interface (in 
   my setup, `enp0s25`)
```
$ ping ff02::1%enp0s25
PING ff02::1%enp0s25(ff02::1%enp0s25) 56 data bytes
64 bytes from fe80::9037:c411:f458:3f0f%enp0s25: icmp_seq=1 ttl=64 time=0.045 ms
64 bytes from fe80::ab4e:c752:3c27:b231%enp0s25: icmp_seq=1 ttl=64 time=0.859 ms
...
```

The address with the longest ping time is the one you want.  In the example above,
that would be `fe80::ab4e:c752:3c27:b231%enp0s25`

3. SSH to the RPI with username `pi` and and no password

```
ssh pi@fe80::ab4e:c752:3c27:b231%enp0s25
pi@raspberrypi-three:~ $ 
```

## Development Drive setup

On your development host, format a USB drive as ext2, copy your github keys, and checkout haxo-rs.
Move the USB drive over to the RPI.
