#!/bin/bash

# Build and run the application
echo "Building the application..."
cargo build

if [ $? -eq 0 ]; then
    echo "Build successful! Starting the application..."
    echo "The application will be available at http://localhost:3000"
    echo "Press Ctrl+C to stop the application"
    cargo run
else
    echo "Build failed. Please check the error messages above."
fi
