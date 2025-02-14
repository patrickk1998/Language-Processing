#!/bin/zsh

# This script processes a raw text corpus through three stages:
# 
# 1. Formatting the corpus: Converts raw text into a formatted version.
# 2. Counting words and generating a token dictionary: Analyzes the formatted corpus
#    and produces a vocabulary encoding.
# 3. Encoding words: Uses the generated token dictionary to encode the corpus.
#
# The script attempts to run the last stage first, and if prerequisites are missing,
# it recursively runs previous stages.


# Stage 1. Format the Corpus
# Prerequisite: corpus_raw
corpus_raw="data/raw/masc_500k_texts"
formatter="zsh data/transform.sh"
corpus_formated="data/formated/masc_500k_formated.txt"

# Stage 2. Count words and generate vocabulary token encodings
# Prerequisite: corpus_formated
corpus_dictionary="data/runtime/token_dictionary"
word_counter="cargo run --bin vocab_count"

# Stage 3. Encode words using generated token dictionary
# Prerequisite: corpus_formated corpus_dictionary
corpus_encoded="data/formated/masc_encoded"
encoder="cargo run --bin binary_encoder"

# Function to format the corpus
format_corpus() {
    if [[ -d "$corpus_raw" ]]; then
        echo "Formatting corpus..."
        eval $formatter "$corpus_raw" "$corpus_formated" || { echo "Corpus formatting failed."; exit 1; }
        generate_dictionary
    else
        echo "Error: Raw corpus directory '$corpus_raw' not found."
        exit 1
    fi
}

# Function to generate token dictionary
generate_dictionary() {
    if [[ -f "$corpus_formated" ]]; then
        echo "Generating token dictionary..."
        eval $word_counter "$corpus_formated" "$corpus_dictionary" || { echo "Dictionary generation failed."; exit 1; } 
        encode_corpus
    else
        echo "Missing formatted corpus. Attempting to format corpus..."
        format_corpus
    fi
}

# Function to encode words using generated token dictionary
encode_corpus() {
    if [[ -f "$corpus_formated" && -f "$corpus_dictionary" ]]; then
        echo "Encoding corpus..."
        eval $encoder "$corpus_dictionary" "$corpus_formated" "$corpus_encoded" || { echo "Encoding failed."; exit 1; }
    else
        echo "Missing prerequisites for encoding. Attempting to generate dictionary..."
        generate_dictionary
    fi
}

encode_corpus

echo "Ngram Pipeline completed successfully."
