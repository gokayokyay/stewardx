docker run --rm -P -p 127.0.0.1:5432:5432 -v "$HOME/postgres-data:/var/lib/postgresql/data" -e POSTGRES_PASSWORD="1234" --name pg postgres:alpine
