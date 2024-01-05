#! /bin/bash
#
# Install haxo service on a SD card running a freshly installed OS
#
# The cards needs to be inserted on a Raspberry Pi that is be accessible via
# ethernet.
#
set -x

# Rasberry Pi local path where haxo-rs will be natively compiled
HAXO_RS_LOCAL_PATH="/media/usb/dev/haxo-rs"

TARGET="haxophone.local"
SSH="ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR"
C="sshpass -p haxophone ${SSH} pi@${TARGET}"

# Wait for device to be live 
function wait_for_target() {
while ! $C uname &> /dev/null
do
    sleep 10
    echo Waiting for target...
done 
echo
}

wait_for_target

# Check that overlay is disabled
OVERLAY_DISABLED=$( $C sudo raspi-config nonint get_overlay_now )
[ "${OVERLAY_DISABLED}" = "1" ] || $C "sudo raspi-config nonint disable_overlayfs; reboot"

wait_for_target

# Check that boot partition is writable
BOOT_IS_WRITABLE=$( $C sudo raspi-config nonint get_bootro_now )
[ "${BOOT_IS_WRITABLE}" = "1" ] || $C "sudo raspi-config nonint disable_bootro; reboot"

wait_for_target

# Enable I2C and serial port
$C sudo raspi-config nonint do_i2c 0  # 0 means enable
$C sudo raspi-config nonint do_serial 0

# Check what type of target this is 
MODEL=$($C cat /proc/cpuinfo | grep Model | sed 's/.*Raspberry Pi \([^R]*\) Rev.*/\1/')

# Trim whitespace
MODEL="${MODEL%"${MODEL##*[![:space:]]}"}"
echo ${MODEL} | grep Zero > /dev/null && IS_ZERO=y || IS_ZERO=n
echo Detected Raspberry Pi Model ${MODEL} \(Is a Zero = $IS_ZERO\)

# Install depedencies 
$C sudo apt-get install -y libfluidsynth-dev git libasound2-dev i2c-tools fluid-soundfont-gm

# Enable config.txt bits
$C tail -1 /boot/config.txt | grep '\[all\]' || $C 'echo \[all\] | sudo tee -a /boot/config.txt'
$C grep 'disable-wifi' /boot/config.txt || $C 'echo dtoverlay=disable-wifi | sudo tee -a /boot/config.txt'
$C grep 'disable-bt' /boot/config.txt || $C 'echo dtoverlay=disable-bt | sudo tee -a /boot/config.txt'
$C grep 'max98357a' /boot/config.txt || $C 'echo dtoverlay=max98357a,sdmode-pin=4 | sudo tee -a /boot/config.txt'
$C grep 'dtparam=i2c_arm_baudrate=400000' /boot/config.txt || $C 'echo dtparam=i2c_arm_baudrate=400000 | sudo tee -a /boot/config.txt'

# Disable config.txt bits
$C grep '^dtparam=audio=on' /boot/config.txt && $C 'sudo sed -i "s/dtparam=audio=on/# dtparam=audio=on/" /boot/config.txt'

# Auto mount USB drive if attached
$C [ -d /media/usb ] || $C sudo mkdir /media/usb
$C grep '/media/usb' /etc/fstab || $C echo '"/dev/sda1       /media/usb        ext4    defaults,nofail        0 2" | sudo tee -a /etc/fstab'
$C sudo mount -a

# Pi Zero specific 
if [ "${IS_ZERO}" = 'y' ]
then
    # Enable USB OTG
    $C grep 'dtoverlay=dwc2' /boot/config.txt || $C 'echo dtoverlay=dwc2 | sudo tee -a /boot/config.txt'
    $C grep 'dwc2' /etc/modules || $C 'echo dwc2 | sudo tee -a /etc/modules'

    # Increase swap file, required for rust installation
    $C sudo dphys-swapfile swapoff
    $C sudo sed -i -e 's/CONF_SWAPSIZE=.*/CONF_SWAPSIZE=512/' /etc/dphys-swapfile
    $C sudo dphys-swapfile setup
    $C sudo dphys-swapfile swapon 
fi

# Install Rust
$C 'curl https://sh.rustup.rs -sSf | sh -s -- -y'
CARGO='/home/pi/.cargo/bin/cargo'

# Try to find sources
$C [ -d "${HAXO_RS_LOCAL_PATH}" ] || { echo "haxo-rs repository not found on target at ${HAXO_RS_LOCAL_PATH}.  Nothing else to do"; exit 0; }

# Compile executable
if [ "${IS_ZERO}" = 'y' ]
then
    echo Compiling with MIDI support
    $C "cd ${HAXO_RS_LOCAL_PATH}; ${CARGO} build --release --features midi"
else
    echo Compiling without MIDI support
    $C "cd ${HAXO_RS_LOCAL_PATH}; ${CARGO} build --release"
fi
# Install
$C "sudo cp ${HAXO_RS_LOCAL_PATH}/target/release/haxo001 /usr/local/bin"
$C "cd ${HAXO_RS_LOCAL_PATH}/scripts/systemd; sudo ./install.sh"

# Disable swap
$C sudo dphys-swapfile swapoff
$C sudo sed -i -e 's/CONF_SWAPSIZE=.*/CONF_SWAPSIZE=0/' /etc/dphys-swapfile
$C sudo dphys-swapfile setup

# Clear haxo logs
$C sudo journalctl --rotate --vacuum-time=1s

# Enable overlay
$C sudo raspi-config nonint enable_overlayfs
$C sudo reboot
echo Done
