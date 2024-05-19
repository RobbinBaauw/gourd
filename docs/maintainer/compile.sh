#!/bin/bash
mkdir out
cd src

compile_xelatex() {
    xelatex -halt-on-error -shell-escape -interaction=nonstopmode -output-directory=../out maintainer.tex
}

compile_xelatex && compile_xelatex
