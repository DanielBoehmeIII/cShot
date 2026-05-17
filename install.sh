#!/usr/bin/env bash
set -e

echo "cShot Installer"
echo "================"
echo ""

# Check Python
PYTHON=$(command -v python3 || command -v python)
if [ -z "$PYTHON" ]; then
    echo "Error: Python 3 not found. Install Python 3.10+ first."
    exit 1
fi

echo "Python: $($PYTHON --version)"

# Check pip
PIP=$(command -v pip3 || command -v pip)
if [ -z "$PIP" ]; then
    echo "Error: pip not found."
    exit 1
fi

# Install dependencies
echo "Installing Python dependencies..."
$PIP install -r requirements.txt

# Check for libsndfile (needed by soundfile)
if command -v apt-get &>/dev/null; then
    dpkg -l libsndfile1 &>/dev/null || {
        echo "Installing libsndfile1 (required by soundfile)..."
        sudo apt-get install -y libsndfile1
    }
elif command -v brew &>/dev/null; then
    brew list libsndfile &>/dev/null || {
        echo "Install libsndfile: brew install libsndfile"
    }
fi

echo ""
echo "Installation complete!"
echo ""
echo "Quick test:"
echo "  ./cshot prompt 'dark soft piano stab' --out outputs/test.wav"
echo ""
echo "Or install as a package:"
echo "  pip install -e ."
echo "  cshot prompt 'dark soft piano stab' --out outputs/test.wav"
echo ""
echo "For the UI:"
echo "  pip install gradio"
echo "  python3 app.py"
