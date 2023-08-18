# Simple GPU Info

A simple tool to get information about GPU utilization, for Linux.

![Example](./img/example.png)

## Usage

```sh
$ simple_gpu_info --help
Simple program to get the GPU usage of a process

Usage: simple_gpu_info [OPTIONS] <--name <NAME>|--loging>

Options:
  -n, --name <NAME>            Name of the a process
  -l, --loging                 Log the GPU usage
  -p, --print-info             Print info about the GPU and the process
  -d, --disable-notification   Disable the notification
  -L, --log-path <LOG_PATH>    Path to the log file [default: /tmp/gpu-usage.log]
  -d, --delimiter <DELIMITER>  Log delimiter [default: ", "]
  -h, --help                   Print help
  -V, --version                Print version
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
