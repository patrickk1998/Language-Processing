#!/usr/bin/env python3
import argparse
import sys

'''
	This script transforms raw text data from corpus into a newline seperated 
	string of words, which is appened to an output file. Each word is either a string of 
	alpha numeric characters seperated by whitespace or punctuation ('.', '!', '-', and ',').

	This script takes about 10 seconds to process 500k words on an Apple Silicon M1 – single threaded of course.
	running multiple processes will speed throughput up linearly until core count is reached since this script is
	CPU constrained. Will probably rewrite in Rust at somepoint in the future.
'''

def parse_args():
    parser = argparse.ArgumentParser(description='Split text into words and write to file')
    parser.add_argument('-i', '--input', default='example.txt',
                      help='Input file path (default: example.txt)')
    parser.add_argument('-o', '--output',
                      help='Output file path (default: stdout)')
    parser.add_argument('--test', action='store_true',
                      help='Enable test mode')
    return parser.parse_args()

def split_into_words(content):
    words = []
    current_word = [] # .join is faster than string concatination
    punct = set('.!,-') 
    
    for char in content:
        if char.isspace():
            if current_word:
                words.append(''.join(current_word).lower())
                current_word = []
        elif char in punct:
            if current_word:
                words.append(''.join(current_word).lower())
                current_word = []
            words.append(char)
        elif char.isalnum():
            current_word.append(char)
    
    # Don't forget the last word if it exists
    if current_word:
        words.append(''.join(current_word))
	
    return words
    
def process_file(input_path, output_file, test = False):
    try:
        with open(input_path, 'r', encoding='utf-8') as f:
            content = f.read()
            
        if test:
            print(f"Input content:\n{content}")
            
        words = split_into_words(content)
        
        if test:
            for w in words:
                if w.isspace():
                    print(f"Whitespace detected: {w}")
                if w == "":
                    print("Empty string as word detected")

        for word in words:
            output_file.write(f"{word}\n")
            
    except FileNotFoundError:
        print(f"Error: Input file '{input_path}' not found", file=sys.stderr)
        sys.exit(1)
    except UnicodeDecodeError:
        print(f"Error: File '{input_path}' is not valid UTF-8", file=sys.stderr)
        sys.exit(1)

args = parse_args() 
output_file = open(args.output, 'a') if args.output else sys.stdout    
process_file(args.input, output_file, args.test)
