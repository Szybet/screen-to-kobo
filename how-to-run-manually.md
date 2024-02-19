This describes the process of running version v0.0.1 manually on InkBox

do it in the order described, if something fails don't go further, it doesn't make sense

if you don't have permissions to execute a file, do `chmod +x thefile`

1. Download files from here: https://github.com/Szybet/screen-to-kobo/releases/tag/v0.0.1
2. Make USBNET to work
3. Run `./sct-host` on your computer
    - Adjust command line parametes
    - You need to have imagemagick installed on your system
    - Tested only on wayland
4. Open 2 terminal windows. In them:
     1. In the first Terminal window do in order:
           - `scp sct-kobo-frontend root@192.168.2.2:/kobo/` - copy the file to the device
           - ssh now into the device
           - execute `chroot /kobo`
           - then `cd /` and `env LD_LIBRARY_PATH=/mnt/onboard/.adds/qt-linux-5.15.2-kobo/lib:/lib/:/usr/lib QT_QPA_PLATFORM=kobo ./sct-kobo-frontend`
     2. In the second terminal window:
           - `scp sct-kobo-backend root@192.168.2.2:/` - copy the file to the device
           - ssh to the device
           - execute `./sct-kobo-backend`. Adjust parameters to make it connect

That's it?
