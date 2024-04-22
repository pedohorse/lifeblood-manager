# Lifeblood Manager

This is a tool made to ease setup and management of [Lifeblood](https://github.com/pedohorse/lifeblood)

## Usage

Check the [releases](https://github.com/pedohorse/lifeblood-manager/releases), download latest lifeblood-manager
for your platform.

Put the file into an empty directory where you want lifeblood to be installed, launch it.

The tool automatically downloads latest commit from given branch of the github repo.

Then it manages a link to one of the versions, called "current". You can easily switch "current" version with the tool.
Easily try newest version, and easily fall back to the previous one if things go wrong.

### Control

#### Environment

* `PYTHON_BIN` env is used to locate python to be used with new versions being installed

## systemd service

This repository also provides a script to automate installation of lifeblood as systemd service,
check [systemd-service](systemd-service) directory for more information