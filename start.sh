#!/bin/sh

APP_MODE=${APP_MODE:-book}

if [ "$APP_MODE" = "book" ]; then
    exec /usr/local/bin/book
elif [ "$APP_MODE" = "tick" ]; then
    exec /usr/local/bin/tick
elif [ "$APP_MODE" = "full" ]; then
    exec /usr/local/bin/full
else
    echo "APP_MODE desconhecido: $APP_MODE"
    exit 1
fi
