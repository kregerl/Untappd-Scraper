#!/bin/bash

if [ -n "$1" ]; then
  PID=$(./driver/geckodriver --port "$1" > /dev/null 2>&1 & echo $!)
  sleep 1
  cargo run --package untapped_scraper --bin untapped_scraper "$1"
  kill "$PID"
  exit 0
else
  echo "You need to specify a port for the webdriver to run on."
  exit 1
fi