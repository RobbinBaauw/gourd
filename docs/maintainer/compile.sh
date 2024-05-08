#!/bin/bash
mkdir out
cd src
pdflatex -halt-on-error -output-directory ../out main.tex && pdflatex -halt-on-error -output-directory ../out main.tex && mv ../out/main.pdf ../../../gourd-maintainer-documentation.pdf
