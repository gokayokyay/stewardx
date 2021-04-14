#!/bin/bash

STEWARDX_HOME=$(pwd)

cd $HOME
if [ -d "$HOME/stewardx-panel" ] 
then
    echo "Directory stewardx-panel exists. Skipping git clone..." 
    cd stewardx-panel
    git pull
else
    echo "Directort stewardx-panel doesn't exists, cloning it..."
    git clone git@github.com:gokayokyay/stewardx-panel.git
    cd stewardx-panel
fi

echo "Cleaning node_modules for a fresh start!"
rm -rf node_modules
echo "Installing the modules..."
npm install
echo "Now building it, this can take a while"
npm run build
echo "Okay, now moving the artifact into the panel directory."
mv dist/index.html $STEWARDX_HOME/panel
echo "Done."