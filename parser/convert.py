"""
Python script for converting parsed .gg files in other programming languages by using rules

Example:
    python convert.py cpp example.txt
"""

from typing import List
import sys


def main(argc: int, argv: List[str]):
    if argc == 1:
        print("No given language to convert and no file to convert chosen")
        sys.exit(-1)
    elif argc == 2:
        print("No file to convert chosen")
        sys.exit(-1)

    convert_file = open(argv[2], mode="r")
    output = ""



    sys.exit(0)


if __name__ == "__main__":  # When executed as script, not imported
    main(len(sys.argv), sys.argv)
