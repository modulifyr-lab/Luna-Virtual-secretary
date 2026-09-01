"""
G2P Phonemization Bridge for Luna using Misaki (fallback when espeak-ng is not available).
"""

import sys

def main():
    if len(sys.argv) > 1:
        text = sys.argv[1]
    else:
        text = sys.stdin.read()

    if not text.strip():
        print("")
        return

    try:
        from misaki import en
        g2p = en.G2P()
        phonemes, _ = g2p(text)
        print(phonemes)
    except Exception as e:
        sys.stderr.write(f"Misaki G2P error: {e}\n")
        print(text)

if __name__ == "__main__":
    main()
