#!/bin/bash
mkdir out
cd src

compile_xelatex() {
    xelatex -halt-on-error -shell-escape -interaction=nonstopmode -output-directory=../out -aux-directory=../out main.tex
}

compile_xelatex && compile_xelatex && mv ../out/main.pdf ../../../gourd-maintainer-documentation.pdf
