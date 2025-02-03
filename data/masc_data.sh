#!/bin/zsh

# This script downloads 500K words of text data from the MASC project into data/raw/masc_500k_texts.
# Information about this text corpus can be found at https://anc.org/data/masc/about/.

curl --output data/raw/masc_500k.tgz https://www.anc.org/MASC/download/masc_500k_texts.tgz
tar -zxvf data/raw/masc_500k.tgz -C data/raw/
rm data/raw/masc_500k.tgz
