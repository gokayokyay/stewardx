#!/bin/bash

### This script clones the temporary panel of StewardX to your HOME directory
### builds it then moves it to directory of STEWARDX_CONFIG

if [[ -z "${STEWARDX_CONFIG}" ]]; then
  STEWARDX_CONFIG_DIR=$HOME/.config/stewardx/
else
  STEWARDX_CONFIG_DIR="$(dirname ${STEWARDX_CONFIG})"
fi

mkdir -p $STEWARDX_CONFIG_DIR

cd $HOME
if [ -d "$HOME/stewardx-panel" ] 
then
    echo "Directory stewardx-panel exists. Skipping git clone..." 
    cd stewardx-panel
    git pull
else
    echo "Directory stewardx-panel doesn't exists, cloning it..."
    git clone https://github.com/gokayokyay/stewardx-panel.git
    cd stewardx-panel
fi

echo "Cleaning node_modules for a fresh start!"
rm -rf node_modules
echo "Installing the modules..."
npm install
echo "Now building it, this can take a while"
npm run build
echo "Okay, now moving the artifact into the panel directory."
mv dist/index.html $STEWARDX_CONFIG_DIR/
echo "Done."