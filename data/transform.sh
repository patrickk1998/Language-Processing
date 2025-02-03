#!/bin/zsh

# This script transforms all .txt files in a directory using ./transform.py and appends 
# the transformation to an output file.

# Function to display usage information
show_usage() {
    echo "Usage: $0 <directory> <file>"
    echo "Example: $0 ~/corpus word_file"
    exit 1
}

# Check if we have the correct number of arguments
if [[ $# -ne 2 ]]; then
    show_usage
fi

directory="$1"
outputFile="$2"
program="./transform.py"

# Check if directory exists
if [[ ! -d "$directory" ]]; then
    echo "Error: Directory '$directory' does not exist."
    exit 1
fi

# Check if program exists and is executable
if ! command -v "$program" &> /dev/null; then
    echo "Error: Program '$program' not found or not executable."
    exit 1
fi


# Process all .txt files recursively. Beacuse of how the builtin zsh time works (it has no -p option) 
# we need use use the time executable timing a separate shell.
code='find "$directory" -type f -name "*.txt" -print0 | while IFS= read -r -d "" file; do
    echo "Processing: $file"
    "$program" "-i" "$file" "-o" "$outputFile" 
done'

command time -p zsh -c "directory=$directory; outputFile=$outputFile; program=$program;  $code"

echo "Processing complete!"
