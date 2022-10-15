#!/bin/bash

# Run cargo in watch mode
cargo watch -q -c -x 'run -q' 

# Run created docker image with env vars from .env
# docker run -p 8080:8080 --name booking-machine-container --env-file .env booking-machine
