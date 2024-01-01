#! /bin/sh
modprobe libcomposite
if [ -d /sys/kernel/config/usb_gadget/midi ]; then
  echo "usb midi gadget already exists" >&2
  exit 0
fi

mkdir /sys/kernel/config/usb_gadget/midi
cd /sys/kernel/config/usb_gadget/midi
mkdir configs/c.1
mkdir functions/midi.usb0
mkdir strings/0x409
mkdir configs/c.1/strings/0x409
echo 0x2BAD > idProduct
echo 0x1209 > idVendor
echo 0001 > strings/0x409/serialnumber
echo CardonaBits > strings/0x409/manufacturer
echo Haxophone > strings/0x409/product
echo "Conf 1" > configs/c.1/strings/0x409/configuration
echo 120 > configs/c.1/MaxPower
ln -s functions/midi.usb0 configs/c.1
echo $(ls /sys/class/udc) > UDC
echo "created usb midi gadget" >&2
