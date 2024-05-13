#!/bin/bash
mkdir out

compile_xelatex() {
    xelatex -halt-on-error -shell-escape -interaction=nonstopmode -output-directory=./out gourd.tex
}

compile_xelatex && compile_xelatex && mv ./out/gourd.pdf ./../../gourd-user-documentation.pdf
latex2man gourd.tex ./../../gourd.man
