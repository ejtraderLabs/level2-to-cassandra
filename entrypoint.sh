#!/bin/sh

APP_MODE=${APP_MODE:-book}

if [ "$APP_MODE" = "book" ]; then
    exec /app/book
elif [ "$APP_MODE" = "tick" ]; then
    exec /app/tick
elif [ "$APP_MODE" = "full" ]; then
    exec /app/full
else
    echo "APP_MODE desconhecido: $APP_MODE"
    exit 1
fi
