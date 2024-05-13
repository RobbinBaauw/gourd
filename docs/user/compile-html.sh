#!/bin/sh
mkdir out

cd out

for manname in gourd.conf gourd
do
    f="../$manname.man"
    cat $f | groff -K utf8 -m man -Thtml > ../$manname-manpage.html
done
