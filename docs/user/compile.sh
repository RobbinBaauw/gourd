#!/bin/bash
mkdir out

compile_xelatex() {
    xelatex -halt-on-error -shell-escape -interaction=nonstopmode -output-directory=./out gourd_pc.tex
}

latex2man -M gourd.tex ./../../gourd.man
latex2man -L gourd.tex ./gourd_pc.tex
compile_xelatex && compile_xelatex
mv ./out/gourd_pc.pdf ../../gourd-user-documentation.pdf
rm gourd_pc.tex
