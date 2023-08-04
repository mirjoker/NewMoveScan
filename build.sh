#!/usr/bin/env bash

cargo build --release 
SHELL_FOLDER=$(cd "$(dirname "$0")";pwd)
RELEASE_PATH=$SHELL_FOLDER"/target/release"
MOVESCANNER_PATH_SOURCE=$RELEASE_PATH"/MoveScanner"
MOVESCANNER_ROOT=$HOME"/.MoveScanner"
MOVESCANNER_BIN=$MOVESCANNER_ROOT"/bin"
MOVESCANNER_PATH_TARGET=$MOVESCANNER_BIN"/MoveScanner"
function LOG_INFO() {
     echo -e "\033[33mINFO: ${1}\033[0m"
}
function LOG_SUCCESS() {
     echo -e "\033[32mSUCCESS: ${1}\033[0m"
}
function LOG_ERROR() {
     echo -e "\033[4m\033[1m\033[31mERROR:\033[0m\033[31m${1}\033[0m"
}
if [ ! -x $MOVESCANNER_PATH_SOURCE ]; then
    LOG_ERROR "please check if 'cargo build --release' success."
else
    if [ ! -d $MOVESCANNER_BIN ]; then
        mkdir -p $MOVESCANNER_BIN
    fi
    /bin/cp -rf $MOVESCANNER_PATH_SOURCE $MOVESCANNER_PATH_TARGET
fi

if [ ! -x $MOVESCANNER_PATH_TARGET ]; then
    LOG_ERROR "please check if 'cargo build --release' success."
else
    LOG_INFO "please add the following to your shell configuration file(.bashrc): \n\t export MOVESCANNER_ROOT=\"\$HOME/.MoveScanner\" \n\t export PATH=\"\$MOVESCANNER_ROOT/bin:\$PATH\""
    LOG_SUCCESS "Start a new terminal session, Try 'MoveScanner -h', enjoy!"
fi