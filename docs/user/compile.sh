#!/bin/bash
mkdir out

compile_xelatex() {
    xelatex -halt-on-error -shell-escape -interaction=nonstopmode -output-directory=./out ./out/$1.tex
}

compile_man_latex() {
    latex2man -M $1.tex ./out/$1.man
    latex2man -L $1.tex ./out/$1.tex
    compile_xelatex $1 && compile_xelatex $1
}

compile_man_latex "gourd"
compile_man_latex "gourd.toml"
