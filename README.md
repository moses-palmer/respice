# Respice

This application is a complement to
[SPICE agent](https://www.linux-kvm.org/page/SPICE) when running a desktop
environment without support for _SPICE_ inside the guest.

This applcation works by listening for the `SCREEN_CHANGE_NOTIFY` event sent by
_XRANDR_, and then resizing the guest display.

Some desktop environments, notably _GNOME_, support this behaviour out of the
box, whereas other more light weight environments do not.
