#!/bin/sh
mkdir out

compile_xelatex() {
    xelatex -halt-on-error -shell-escape -interaction=nonstopmode -output-directory=./out gourd.tex
}

compile_xelatex && compile_xelatex
