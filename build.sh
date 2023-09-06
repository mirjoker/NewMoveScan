#!/usr/bin/env bash

SHELL_FOLDER=$(
    cd "$(dirname "$0")"
    pwd
)
RELEASE_PATH=$SHELL_FOLDER"/target/release"
MOVESCANNER_PATH_SOURCE=$RELEASE_PATH"/MoveScanner"
MOVESCANNER_ROOT=$HOME"/.MoveScanner"
MOVESCANNER_BIN=$MOVESCANNER_ROOT"/bin"
MOVESCANNER_PATH_TARGET=$MOVESCANNER_BIN"/MoveScanner"
DEPENDENCIES=("move-binary-format" "move-core-types" "move-model" "move-package" "move-stackless-bytecode")

function LOG_INFO() {
    echo -e "\033[33mINFO: ${1}\033[0m"
}
function LOG_SUCCESS() {
    echo -e "\033[32mSUCCESS: ${1}\033[0m"
}
function LOG_ERROR() {
    echo -e "\033[4m\033[1m\033[31mERROR:\033[0m\033[31m${1}\033[0m"
}
function CHANGE_MOVE() {
    local param="$1"
    for dep in "${DEPENDENCIES[@]}"; do
        echo "$param"
        if [ "$param" == "move20" ]; then
            cargo remove $dep
            cargo add $dep --git https://github.com/uestc-sec/move --branch default
        elif [ "$param" == "move32" ]; then
            cargo remove $dep
            cargo add $dep --git https://github.com/uestc-sec/move --branch main
        else
            LOG_ERROR "Please use correct build option"
            exit 1
        fi
    done
}

if [ $# -eq 1 ]; then
    CHANGE_MOVE "$1"
fi

cargo build --release

if [ ! -x $MOVESCANNER_PATH_SOURCE ]; then
    LOG_ERROR "Please check if 'cargo build --release' success."
else
    if [ ! -d $MOVESCANNER_BIN ]; then
        mkdir -p $MOVESCANNER_BIN
    fi
    /bin/cp -rf $MOVESCANNER_PATH_SOURCE $MOVESCANNER_PATH_TARGET
fi

config_flag=false
function TestIfConfiged() {
    if [ -e ${1} ]; then
        if [ $(grep -c "export MOVESCANNER_ROOT=\"\$HOME/.MoveScanner\"" ${1}) -ne "0" ] && [ $(grep -c "export PATH=\"\$MOVESCANNER_ROOT/bin:\$PATH\"" ${1}) -ne "0" ]; then
            config_flag=true
            LOG_SUCCESS "Find configuration in ${1}."
        fi
    fi
}

function DetectAndConfigShell() {
    shell_type=$(echo $SHELL)
    if echo "$shell_type" | grep -q "zsh"; then
        shell_config=$HOME/.zshrc
    elif echo "$shell_type" | grep -q "bash"; then
        shell_config=$HOME/.bahrc
    elif echo "$shell_type" | grep -q "fish"; then
        shell_config=$HOME/.config/fish/config.fish
    fi
    if [ -e $shell_config ]; then
        echo 'export MOVESCANNER_ROOT="$HOME/.MoveScanner"' >>$shell_config
        echo 'export PATH="$MOVESCANNER_ROOT/bin:$PATH"' >>$shell_config
        LOG_SUCCESS "Detected that your default shell is $shell_type, MoveScanner has been automatically set for you."
        LOG_SUCCESS "Start a new terminal session, Try 'MoveScanner -h', enjoy!"
        LOG_INFO "If you wish to use MoveScanner on another shell, please add the following to the shell's configuration file(e.g. ~/.bashrc, ~/.zshrc): \n\t export MOVESCANNER_ROOT=\"\$HOME/.MoveScanner\" \n\t export PATH=\"\$MOVESCANNER_ROOT/bin:\$PATH\""
    else
        LOG_INFO "Please add the following to your shell configuration file(e.g. ~/.bashrc, ~/.zshrc): \n\t export MOVESCANNER_ROOT=\"\$HOME/.MoveScanner\" \n\t export PATH=\"\$MOVESCANNER_ROOT/bin:\$PATH\""
    fi
}

if [ ! -x $MOVESCANNER_PATH_TARGET ]; then
    LOG_ERROR "No executable found in $MOVESCANNER_BIN."
else
    TestIfConfiged $HOME/.bashrc
    TestIfConfiged $HOME/.zshrc
    TestIfConfiged $HOME/.config/fish/config.fish
    TestIfConfiged $HOME/.bash_profile
    TestIfConfiged $HOME/.profile
    TestIfConfiged /etc/bashrc
    TestIfConfiged /etc/profile
    if [ $config_flag = true ]; then
        LOG_SUCCESS "Start a new terminal session, Try 'MoveScanner -h', enjoy!"
    else
        DetectAndConfigShell
    fi
fi
