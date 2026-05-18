#!/usr/bin/env python3
"""
cShot — custom one-shot kits from songs, samples, genres, and vibes.

Usage:
  ./cshot make "dark rnb one shot kit"     Generate a complete kit from a description
  ./cshot from-song song.wav               Generate a kit from a song
  ./cshot from-sample sample.wav           Generate a kit from a sample
  ./cshot lab <command>                    Advanced research commands
"""
from gen.cli import main

if __name__ == "__main__":
    main()
