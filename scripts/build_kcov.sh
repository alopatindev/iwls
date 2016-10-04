#!/bin/sh

wget "https://github.com/SimonKagstrom/kcov/archive/master.tar.gz"
tar xzf "master.tar.gz"
mv "kcov-master" kcov
cd kcov
mkdir build
cd build
cmake ..
make -j2
cd ../..
