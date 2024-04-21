## systemd service installer

These files help automate installation of lifeblood's scheduler and worker components
on headless systems, using latest `lifeblood-manager-cli` and `systemd`

- You need to download all files from this directory (except this `README.md`)
- You may or may not adjust settings in `CONFIG.ME` file
- run `sudo ./install.sh scheduler` to install lifeblood scheduler and a systemd service to manage it
- run `sudo ./install.sh worker` to install lifeblood worker and a systemd service to manage it

### Before install

Check and adjust provided `CONFIG.ME` file.

For most of options - see section below, or check `install.sh` file itself. You can override any variable from what is set in the top.

Also note, that scheduler and worker are **intentionally limited** in what they can access in the system - it's security and just common sense.

Therefore you will need to list what scheduler and worker will be able to access. In default `CONFIG.ME` file you will see lines added to both
service configurations, smth like `ReadWritePaths=-/mnt`. This allows read-write access to `/mnt` subtree, assuming you have some shared directories
mounted there.

In your (home) studio you might have some kind of common mount with all the projects, like `/PROJECTS` - you need to add that to service configurations
instead of (or together with) defaults.  
Like, for example:

```
ReadWritePaths=-/PROJECTS
```

### What exactly does `install.sh` do

1. it will check for existing system user and group `lifeblood` and create those if they are missing
  - You can change that default user name by adding `SERVICE_USER=something` and `SERVICE_GROUP=something` to `[all.install]` section in `CONFIG.ME`
2. it will create directory `lifeblood` under `/opt` (if it does not exist yet)
  - You can change those by adding `INSTALL_BASE=/my/base/path` and `INSTALL_DIR_NAME=deathblood` to `[all.install]` section in `CONFIG.ME`
3. in directory from previous step (`/opt/lifeblood` by default) it will download latest `lifeblood-manager-cli` from github releases
4. it will run `lifeblood-manager-cli` to install latest lifeblood version from `dev` branch
  - You can change the branch by adding `LIFEBLOOD_BRANCH=some-other-branch` to `[all.install]` section in `CONFIG.ME`
5. it will fill out template variables in `lifeblood.service.template` file with correct values and save it to a temporary `lifeblood-<component>.service` file in the same directory
6. it will add all lines as is from `CONFIG.ME` file, from a section, corresponding to current component's service directly to the bottom of the service file.
   So for, example, if you are running `./install.sh scheduler` - lines from section `[scheduler.service]` in `CONFIG.ME` file will be added to the end of generated
   `lifeblood-scheduler.service` file.
7. it will move generated `lifeblood-<component>.service` file to `/etc/systemd/system/` directory (overriding existing if any)
8. it will run `systemctl daemon-reload` to tell systemd to reload unit configurations
9. it will run `systemctl enable <service_name>` to set created service to start automatically with the system
10. it will run `systemctl restart <service_name>` to (re)start service now.

After installation you can check if service is running with

```
systemctl status lifeblood-scheduler.service
systemctl status lifeblood-worker.service
```

You can get full logs of scheduler and worker with 
```
journalctl -u lifeblood-scheduler.service
journalctl -u lifeblood-worker.service
```

> [!NOTE]
> You mignt need to run commands above with `sudo`, depending on your system configuration

### About systemd hardening

Currently provided systemd service file is somewhat hardened.

- unnecessary system calls are disabled
- access to where it should not have access is disabled
- access to some namespaces is disabled

However, I'm not a specialist in systemd hardening, so suggestions are welcomed

I'll just add a note that:
- user and cgroup namespace creation is allowed, as it is quite common for environment resolvers to run jobs as given users, 
  and impose resource restrictions on generated processes,
  but currently this functional is not used.
- note that default line in `CONFIG.ME` adds r/w access to `/mnt` subtree, as an example of how to add access to your own shared studio mount,
  you should change that to something appropriate