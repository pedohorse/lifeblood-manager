#!/usr/bin/env bash

set -e -o pipefail

COMPONENT="$1"

# defaults, may be overriden with CONFIG.ME file
SERVICE_USER=lifeblood
SERVICE_GROUP=$SERVICE_USER
INSTALL_BASE=/opt
INSTALL_DIR_NAME=lifeblood

SERVICE_TEMPLATE_FILE=lifeblood.service.template
OVERRIDE_CONFIG_FILE=CONFIG.ME

MANAGER_REPO=pedohorse/lifeblood-manager
LIFEBLOOD_BRANCH=dev

# override parameters

if [[ -f "$OVERRIDE_CONFIG_FILE" ]]; then
    while read -r LINE; do
        if [[ -z "$LINE" || "$LINE" =~ ^\s*#.* ]]; then
            continue
        fi
        if [[ "$LINE" =~ ^\s*\[(.*)\.(.+)\]\s*$ ]]; then
            c_component=${BASH_REMATCH[1]}
            c_stage=${BASH_REMATCH[2]}
            continue
        fi

        if [[ "$c_component" == "$COMPONENT" || "$c_component" == "all" ]] && [[ "$c_stage" == "install" ]]; then
            eval "$LINE"
        fi
    done < "$OVERRIDE_CONFIG_FILE"
fi

# checks

if [[ -z "$COMPONENT" ]]; then
    echo "component must be provided: scheduler, worker"
    exit 2
fi

if [[ "$COMPONENT" == "scheduler" ]]; then
    SERVICE_NAME=lifeblood-scheduler.service
elif [[ "$COMPONENT" == "worker" ]]; then
    SERVICE_NAME=lifeblood-worker.service
else
    echo "component must be one of: scheduler, worker"
    echo "instead $COMPONENT was given"
    exit 2
fi

if ! which systemd-run > /dev/null; then
    echo "system does not have systemd, abort"
    exit 1
fi

if ! which wget >/dev/null 2>&1; then
    echo "wget not found. it is needed to download fresh lifeblood-manager"
    exit 1
fi

if [[ ! -d "$INSTALL_BASE" ]]; then
    echo "install base directory $INSTALL_BASE does not exist, abort"
    exit 1
fi

if [[ ! -f "$SERVICE_TEMPLATE_FILE" ]]; then
    echo "required additional file $SERVICE_TEMPLATE_FILE is not found"
    exit 1
fi

set +e
if [[ -z "$PYTHON_BIN" ]]; then
    PYTHON_BIN=$(which python)
fi
if [[ -z "$PYTHON_BIN" ]]; then
    PYTHON_BIN=$(which python3)
fi
set -e
if [[ -z "$PYTHON_BIN" ]]; then
    echo "python not found! you can provide path to custom python with PYTHON_BIN env"
    exit 1
fi
if ! "$PYTHON_BIN" -m venv -h 2>&1 1>/dev/null; then
    echo "venv python module not found. this setup relies on venv to isolate python envs"
    echo "on debian/ubuntu systems you can install venv with something like:"
    echo "    apt install python3-venv"
    exit 1
fi
if ! "$PYTHON_BIN" -m ensurepip -h 2>&1 1>/dev/null; then
    echo "ensurepip python module not found. this setup relies on pip to get lifeblood dependencies"
    echo "on debian/ubuntu systems you can install ensurepip"
    echo "as part of venv package with something like:"
    echo "    apt install python3-venv"
    exit 1
fi
echo python found at "$PYTHON_BIN"

if (( $UID != 0 )); then
    echo "install script must be run as root to do the following:"
    echo "- create system user $SERVICE_USER for the service if needed"
    echo "- create files under $INSTALL_DIR and change their ownership to $SERVICE_USER"
    echo "- install systemd service to run Lifeblood scheduler and/or worker pool"
    echo ""
    echo "if you want to continue - rerun this script with sudo"
    exit 1
fi

user_exists=1
if id -u $SERVICE_USER >/dev/null; then
    user_exists=0
fi
group_exists=1
if id -g $SERVICE_GROUP >/dev/null; then
    group_exists=0
fi

# if user exists but group doesnt, or the other way around - it's unexpected, something is wrong
if [[ $user_exists == 0 && $group_exists != 0 || $user_exists != 0 && $group_exists == 0 ]]; then
    echo "given user:group pair partially exist, which is unexpected"
    echo "i better fail and let you resolve the situation"
    exit 1
fi

# install

# create service user
if [[ $user_exists != 0 ]]; then
    echo "creating user $SERVICE_USER:$SERVICE_GROUP"
    groupadd --system "$SERVICE_GROUP"
    useradd -g "$SERVICE_GROUP" -N --system "$SERVICE_USER"
fi

# install the stuff
install_dir="$INSTALL_BASE"/"$INSTALL_DIR_NAME"
if [[ ! -d "$install_dir" ]]; then
    mkdir "$install_dir"
fi

chown "$SERVICE_USER:$SERVICE_GROUP" "$install_dir"

pushd "$install_dir"

# here we use su, as we do not expect sudo to be installed
# is it too harsh of an assumption?
echo "downloading latest lifeblood-manager"
if [[ -f lifeblood-manager-cli ]]; then
    rm lifeblood-manager-cli
fi
su -c "wget -O lifeblood-manager-cli https://github.com/${MANAGER_REPO}/releases/latest/download/lifeblood-manager-cli" $SERVICE_USER
chmod 544 lifeblood-manager-cli

echo "checking for newest lifeblood commit"
su -c "PYTHON_BIN=$PYTHON_BIN ./lifeblood-manager-cli installs new --branch $LIFEBLOOD_BRANCH --no-viewer ." $SERVICE_USER

popd

echo "installing systemd service"

if [[ "$COMPONENT" == "scheduler" ]]; then
    exec_args='scheduler --db-path ${STATE_DIRECTORY}/scheduler.db'
elif [[ "$COMPONENT" == "worker" ]]; then
    exec_args='pool simple'
else
    echo "bad component: $COMPONENT"
    exit 1
fi

# note - i'm too lazy to escape stuff in vars below... hope it's not there
$PYTHON_BIN -c '
with open("'${SERVICE_TEMPLATE_FILE}'") as f:
    text = f.read()
text = text.format(
    install_dir="'"$install_dir"'",
    service_user="'"$SERVICE_USER"'",
    exec_args="'"$exec_args"'",
)
with open("'"$SERVICE_NAME"'", "w") as f:
    f.write(text)
'

# override parameters
if [[ -f "$OVERRIDE_CONFIG_FILE" ]]; then
    echo "" >> "$SERVICE_NAME"

    while read -r LINE; do
        if [[ -z "$LINE" || "$LINE" =~ ^\s*#.* ]]; then
            continue
        fi
        if [[ "$LINE" =~ ^\s*\[(.*)\.(.+)\]\s*$ ]]; then
            c_component=${BASH_REMATCH[1]}
            c_stage=${BASH_REMATCH[2]}
            continue
        fi

        if [[ "$c_component" == "$COMPONENT" || "$c_component" == "all" ]] && [[ "$c_stage" == "service" ]]; then
            echo "$LINE" >> "$SERVICE_NAME"
        fi
    done < "$OVERRIDE_CONFIG_FILE"
fi

# install service
mv "$SERVICE_NAME" /etc/systemd/system/.

echo "reloading systemd configuration"
systemctl daemon-reload

echo "enabling service auto start. disable it with systemctl disable $SERVICE_NAME"
systemctl enable "$SERVICE_NAME"

echo "(re)starting $SERVICE_NAME"
systemctl restart "$SERVICE_NAME"

echo "sorta kinda done"

