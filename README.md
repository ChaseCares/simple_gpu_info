# Simple GPU Info

A simple tool to get information about GPU utilization, for Linux.

## Usage

```sh
simple_gpu_info --name=plasmashell
```

## Systemd Service and Timer

To run the script with a specific interval, use a systemd timer. Example files [here](config/systemd/user).

Steps:

1. `mkdir -p ~/config/systemd/user`
2. Copy the service and timer files from this repository [here](config/systemd/user)
3. Modify `~/config/systemd/user/simple_gpu_info.timer` to be the desired interval and **add the full path to the executable**
4. Run `systemctl --user enable simple_gpu_info.service`
5. Run `systemctl --user enable simple_gpu_info.timer`

Troubleshooting:

Run `systemctl --user list-timers --all` to view all the current user timers

Run `systemctl --user daemon-reload` to reload the systemd user daemon

Run `journalctl -e --user` to view the systemd user logs

## Works Cited

[nvml-wrapper for GPU info](https://github.com/Cldfire/nvml-wrapper)

[Implementation of nvml-wrapper](https://github.com/BDHU/gpuinfo)

[Notifications](https://github.com/hoodie/notify-rust)

[Systemd User](https://wiki.archlinux.org/title/Systemd/User)

[Systemd Timers](https://wiki.archlinux.org/title/Systemd/Timers)
