#!/bin/bash
set -e

# Detect operating system or target
OS_SUFFIX=""

if [[ "$TARGET" == "x86_64-unknown-linux-musl" ]]; then
    OS_SUFFIX="-linux_musl"
    echo "📦 Detected MUSL target"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS_SUFFIX="-darwin"
    echo "🍎 Detected macOS platform"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS_SUFFIX="-linux_gnu"
    echo "🐧 Detected Linux GNU platform"
else
    echo "⚠️ Unknown platform or target"
    OS_SUFFIX="-unknown"
fi

# Create packages directory if it doesn't exist
if [ ! -d "./packages" ]; then
    echo "📦 Create folder ./packages"
    mkdir -p ./packages
fi

# Clean packages directory
echo "📦 Clean folder ./packages"
rm -rf ./packages/*

# Compile and package each module
for dir in ./modules/*/; do
    if [ -d "$dir" ]; then
        echo "📦 Packing $dir"

        if [[ -n "$TARGET" ]]; then
            cargo run --release -p phlow-runtime --target "$TARGET" -- --package "$dir"
        else
            cargo run --release -p phlow-runtime -- --package "$dir"
        fi

        # Rename tar.gz files to include OS suffix
        for tarfile in *.tar.gz; do
            filename="${tarfile%.tar.gz}"
            echo "📦 Renaming $tarfile to ${filename}${OS_SUFFIX}.tar.gz"
            mv "$tarfile" "${filename}${OS_SUFFIX}.tar.gz"
        done

        # Move to packages folder
        mv *.tar.gz ./packages/
    fi
done
