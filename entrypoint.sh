#!/bin/sh

APP_MODE=${APP_MODE:-book}

if [ "$APP_MODE" = "book" ]; then
    exec /code/book
elif [ "$APP_MODE" = "tick" ]; then
    exec /code/tick
elif [ "$APP_MODE" = "full" ]; then
    exec /code/full
else
    echo "APP_MODE desconhecido: $APP_MODE"
    exit 1
fi
